use std::collections::HashMap;

use egui::{
    epaint::{ImageDelta, Primitive},
    Color32, ImageData, Mesh, Pos2, Rect, TextureId, TexturesDelta,
};
use egui_kittest::TestRenderer;
use image::{Rgba, RgbaImage};

#[derive(Default)]
pub(super) struct SoftwareRenderer {
    textures: HashMap<TextureId, TextureImage>,
}

#[derive(Clone)]
enum TextureImage {
    Color {
        size: [usize; 2],
        pixels: Vec<Color32>,
    },
    Font {
        size: [usize; 2],
        pixels: Vec<f32>,
    },
}

impl TestRenderer for SoftwareRenderer {
    fn handle_delta(&mut self, delta: &TexturesDelta) {
        for (id, image_delta) in &delta.set {
            self.set_texture(*id, image_delta);
        }
        for id in &delta.free {
            self.textures.remove(id);
        }
    }

    fn render(
        &mut self,
        ctx: &egui::Context,
        output: &egui::FullOutput,
    ) -> Result<RgbaImage, String> {
        let pixels_per_point = ctx.pixels_per_point();
        let size = ctx.screen_rect().size() * pixels_per_point;
        let width = size.x.round().max(1.0) as u32;
        let height = size.y.round().max(1.0) as u32;
        let mut image = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        for primitive in ctx.tessellate(output.shapes.clone(), pixels_per_point) {
            if let Primitive::Mesh(mesh) = primitive.primitive {
                self.paint_mesh(&mut image, &mesh, primitive.clip_rect, pixels_per_point);
            }
        }

        Ok(image)
    }
}

impl SoftwareRenderer {
    fn set_texture(&mut self, id: TextureId, delta: &ImageDelta) {
        match delta.pos {
            None => {
                self.textures.insert(id, TextureImage::from(&delta.image));
            }
            Some(pos) => {
                let patch = TextureImage::from(&delta.image);
                if let Some(texture) = self.textures.get_mut(&id) {
                    texture.apply_patch(pos, &patch);
                } else {
                    self.textures.insert(id, patch);
                }
            }
        }
    }

    fn paint_mesh(
        &self,
        image: &mut RgbaImage,
        mesh: &Mesh,
        clip_rect: Rect,
        pixels_per_point: f32,
    ) {
        let clip = PixelClip::from_rect(clip_rect, pixels_per_point, image.width(), image.height());
        if clip.is_empty() {
            return;
        }

        for [a, b, c] in mesh.triangles() {
            let vertices = [
                mesh.vertices[a as usize],
                mesh.vertices[b as usize],
                mesh.vertices[c as usize],
            ];
            self.paint_triangle(image, mesh.texture_id, vertices, clip, pixels_per_point);
        }
    }

    fn paint_triangle(
        &self,
        image: &mut RgbaImage,
        texture_id: TextureId,
        vertices: [egui::epaint::Vertex; 3],
        clip: PixelClip,
        pixels_per_point: f32,
    ) {
        let points = vertices.map(|vertex| point_to_pixel(vertex.pos, pixels_per_point));
        let bounds = triangle_bounds(points, clip);
        if bounds.is_empty() {
            return;
        }

        let area = edge(points[0], points[1], points[2]);
        if area.abs() <= f32::EPSILON {
            return;
        }

        for y in bounds.min_y..bounds.max_y {
            for x in bounds.min_x..bounds.max_x {
                let point = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
                let weights = [
                    edge(points[1], points[2], point) / area,
                    edge(points[2], points[0], point) / area,
                    edge(points[0], points[1], point) / area,
                ];
                if weights.iter().all(|weight| *weight >= -0.0001) {
                    let color = shade_pixel(&vertices, weights, self.textures.get(&texture_id));
                    blend_pixel(image.get_pixel_mut(x, y), color);
                }
            }
        }
    }
}

impl TextureImage {
    fn from(image: &ImageData) -> Self {
        match image {
            ImageData::Color(image) => Self::Color {
                size: image.size,
                pixels: image.pixels.clone(),
            },
            ImageData::Font(image) => Self::Font {
                size: image.size,
                pixels: image.pixels.clone(),
            },
        }
    }

    fn apply_patch(&mut self, [offset_x, offset_y]: [usize; 2], patch: &Self) {
        match (self, patch) {
            (
                Self::Color { size, pixels },
                Self::Color {
                    size: patch_size,
                    pixels: patch_pixels,
                },
            ) => patch_pixels_2d(size, pixels, [offset_x, offset_y], patch_size, patch_pixels),
            (
                Self::Font { size, pixels },
                Self::Font {
                    size: patch_size,
                    pixels: patch_pixels,
                },
            ) => patch_pixels_2d(size, pixels, [offset_x, offset_y], patch_size, patch_pixels),
            (target, patch) => *target = patch.clone(),
        }
    }

    fn sample(&self, uv: Pos2) -> [f32; 4] {
        match self {
            Self::Color { size, pixels } => {
                let color = pixels[texture_index(*size, uv)];
                rgba_to_unit(color)
            }
            Self::Font { size, pixels } => {
                let coverage = pixels[texture_index(*size, uv)].clamp(0.0, 1.0).powf(0.55);
                [coverage, coverage, coverage, coverage]
            }
        }
    }
}

#[derive(Clone, Copy)]
struct PixelClip {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl PixelClip {
    fn from_rect(rect: Rect, pixels_per_point: f32, width: u32, height: u32) -> Self {
        Self {
            min_x: (rect.min.x * pixels_per_point).floor().max(0.0) as u32,
            min_y: (rect.min.y * pixels_per_point).floor().max(0.0) as u32,
            max_x: ((rect.max.x * pixels_per_point).ceil() as u32).min(width),
            max_y: ((rect.max.y * pixels_per_point).ceil() as u32).min(height),
        }
    }

    fn is_empty(self) -> bool {
        self.min_x >= self.max_x || self.min_y >= self.max_y
    }
}

fn patch_pixels_2d<T: Clone>(
    size: &[usize; 2],
    pixels: &mut [T],
    [offset_x, offset_y]: [usize; 2],
    patch_size: &[usize; 2],
    patch_pixels: &[T],
) {
    let [width, _height] = *size;
    let [patch_width, patch_height] = *patch_size;
    for row in 0..patch_height {
        let target_start = (offset_y + row) * width + offset_x;
        let patch_start = row * patch_width;
        let target = &mut pixels[target_start..target_start + patch_width];
        let patch = &patch_pixels[patch_start..patch_start + patch_width];
        target.clone_from_slice(patch);
    }
}

fn point_to_pixel(point: Pos2, pixels_per_point: f32) -> Pos2 {
    Pos2::new(point.x * pixels_per_point, point.y * pixels_per_point)
}

fn triangle_bounds(points: [Pos2; 3], clip: PixelClip) -> PixelClip {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(clip.min_x as f32) as u32;
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(clip.min_y as f32) as u32;
    let max_x = ((points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .min(clip.max_x))
    .max(min_x);
    let max_y = ((points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as u32)
        .min(clip.max_y))
    .max(min_y);

    PixelClip {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

fn edge(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn shade_pixel(
    vertices: &[egui::epaint::Vertex; 3],
    weights: [f32; 3],
    texture: Option<&TextureImage>,
) -> [f32; 4] {
    let mut color = [0.0; 4];
    let mut uv = Pos2::ZERO;
    for (vertex, weight) in vertices.iter().zip(weights) {
        let rgba = rgba_to_unit(vertex.color);
        for channel in 0..4 {
            color[channel] += rgba[channel] * weight;
        }
        uv += vertex.uv.to_vec2() * weight;
    }

    let texture = texture.map_or([1.0, 1.0, 1.0, 1.0], |texture| texture.sample(uv));
    [
        color[0] * texture[0],
        color[1] * texture[1],
        color[2] * texture[2],
        color[3] * texture[3],
    ]
}

fn rgba_to_unit(color: Color32) -> [f32; 4] {
    [
        f32::from(color.r()) / 255.0,
        f32::from(color.g()) / 255.0,
        f32::from(color.b()) / 255.0,
        f32::from(color.a()) / 255.0,
    ]
}

fn texture_index([width, height]: [usize; 2], uv: Pos2) -> usize {
    let x = (uv.x * (width.saturating_sub(1)) as f32)
        .round()
        .clamp(0.0, width.saturating_sub(1) as f32) as usize;
    let y = (uv.y * (height.saturating_sub(1)) as f32)
        .round()
        .clamp(0.0, height.saturating_sub(1) as f32) as usize;
    y * width + x
}

fn blend_pixel(pixel: &mut Rgba<u8>, source: [f32; 4]) {
    let dst_alpha = f32::from(pixel[3]) / 255.0;
    let src_alpha = source[3].clamp(0.0, 1.0);
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    let mut out = [0_u8; 4];

    for channel in 0..3 {
        let dst = f32::from(pixel[channel]) / 255.0;
        let premultiplied = source[channel].clamp(0.0, 1.0) + dst * dst_alpha * (1.0 - src_alpha);
        let straight = if out_alpha > 0.0 {
            premultiplied / out_alpha
        } else {
            0.0
        };
        out[channel] = (straight.clamp(0.0, 1.0) * 255.0).round() as u8;
    }

    out[3] = (out_alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    *pixel = Rgba(out);
}
