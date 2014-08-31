use gfx;
use device;
use freetype as ft;
use cgmath::{Vector3, Matrix3, Matrix4, Matrix, FixedArray, Rad};
use gfx::{Device, DeviceHelper};
use piston::AssetStore;

static VERTEX_SRC: gfx::ShaderSource = shaders! {
    GLSL_120: b"
#version 120

uniform mat4 projection, view;

attribute vec3 pos, color;

varying vec4 v_Color;

void main() {
        v_Color = vec4(color, 1.0);
        gl_Position = projection * view * vec4(pos, 1.0);
}
"
    GLSL_150: b"
#version 150

uniform mat4 projection, view;

in vec3 pos, color;

out vec4 v_Color;

void main() {
        v_Color = vec4(color, 1.0);
        gl_Position = projection * view * vec4(pos, 1.0);
}
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
    GLSL_120: b"
#version 120
varying vec4 v_Color;
void main() {
        gl_FragColor = v_Color;
}
"

    GLSL_150: b"
#version 150
in vec4 v_Color;
out vec4 o_Color;
void main() {
        o_Color = v_Color;
}
"
};

static TEXT_VERTEX_SRC: gfx::ShaderSource = shaders! {
    GLSL_120: b"
#version 120
attribute vec4 coord;
varying vec2 texpos;

void main() {
        gl_Position = vec4(coord.xy, 1, 1);
        texpos = coord.zw;
}
"
    GLSL_150: b"
#version 150
in vec4 coord;
out vec2 texpos;

void main() {
        gl_Position = vec4(coord.xy, 1, 1);
        texpos = coord.zw;
}
"
};

static TEXT_FRAGMENT_SRC: gfx::ShaderSource = shaders! {
    GLSL_120: b"
#version 120
varying vec2 texpos;
uniform sampler2D tex;
uniform vec4 color;

void main() {
        gl_FragColor = vec4(1, 1, 1, texture2D(tex, texpos).a) * color;
}
"
    GLSL_150: b"
#version 150
in vec2 texpos;
uniform sampler2D tex;
uniform vec4 color;
out vec4 o_Color;

void main() {
        o_Color = vec4(1, 1, 1, texture(tex, texpos).a) * color;
}
"
};

#[shader_param(NormalBatch)]
pub struct ShaderParams {
    pub projection: [[f32, ..4], ..4],
    pub view: [[f32, ..4], ..4],
}

#[shader_param(TextBatch)]
pub struct TextShaderParams {
    tex: gfx::shade::TextureParam,
    color: [f32, ..4],
}

#[vertex_format]
struct Vertex {
    pos: [f32, ..3],
    color: [f32, ..3],
}

#[vertex_format]
struct TextVertex {
    coord: [f32, ..4],
}

pub struct Renderer {
    graphics: gfx::Graphics<device::GlDevice, device::GlCommandBuffer>,
    program: device::ProgramHandle,
    text_program: device::ProgramHandle,
    state: gfx::DrawState,
    frame: gfx::Frame,
    freetype: ft::Library,
    face: ft::Face,
}

static ORIG_VECS: [Vector3<f32>, ..3] = [
    Vector3 { x: 0.5, y: 0.0, z: 0.0 },
    Vector3 { x: 0.0, y: 0.5, z: 0.0 },
    Vector3 { x: -0.5, y: 0.0, z: 0.0 },
];

impl Renderer {
    pub fn new(assets: AssetStore, (mut device, frame): (device::GlDevice, gfx::Frame)) -> Renderer {
        let program = device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
        let text_program = device.link_program(TEXT_VERTEX_SRC.clone(),
                                               TEXT_FRAGMENT_SRC.clone()).unwrap();
        let freetype = ft::Library::init().unwrap();
        let face = freetype.new_face(assets.path("Arial.ttf").unwrap().as_str().unwrap(), 0).unwrap();
        face.set_pixel_sizes(0, 48).unwrap();
        Renderer {
            graphics: gfx::Graphics::new(device),
            program: program,
            text_program: text_program,
            state: gfx::DrawState::new().blend(gfx::BlendAlpha),
            frame: frame,
            freetype: freetype,
            face: face,
        }
    }

    pub fn render(&mut self, t: f32) {
        let rot: Matrix3<f32> = Matrix3::from_euler(Rad { s: 0.0 }, Rad { s: 0.0 }, Rad { s: t });

        let vertex_data: Vec<Vertex> = ORIG_VECS.as_slice().iter().enumerate().map(|(i, v)| {
            Vertex {
                pos: rot.mul_v(v).into_fixed(),
                color: match i {
                    0 => [1.0, 0.0, 0.0],
                    1 => [0.0, 1.0, 0.0],
                    _ => [0.0, 0.0, 1.0],
                }
            }
        }).collect();
        let mesh = self.graphics.device.create_mesh(vertex_data);
        let batch = self.graphics.make_batch(&mesh, mesh.get_slice(gfx::TriangleList),
                                             &self.program, &self.state).unwrap();
        self.graphics.clear(
            gfx::ClearData {
                color: Some([0.3, 0.3, 0.3, 0.1]),
                depth: None,
                stencil: None,
            },
            &self.frame
                );

        let params = ShaderParams {
            projection: Matrix4::identity().into_fixed(),
            view: Matrix4::identity().into_fixed(),
        };

        self.graphics.draw(&batch, &params, &self.frame);

        self.render_text("hello", 0.003, 0.003);

        self.graphics.end_frame();
    }

    fn render_text(&mut self, text: &str, sx: f32, sy: f32) {
        let mut x = 0.0;
        let mut y = 0.0;
        for ch in text.chars() {
            self.face.load_char(ch as u64, ft::face::Render).unwrap();
            let g = self.face.glyph();
            let bitmap = g.bitmap();

            let texture_info = gfx::tex::TextureInfo {
                width: bitmap.width() as u16,
                height: bitmap.rows() as u16,
                depth: 1,
                levels: 1,
                kind: gfx::tex::Texture2D,
                format: gfx::tex::RGBA8,
            };
            let image_info = texture_info.to_image_info();
            let texture = self.graphics.device.create_texture(texture_info).unwrap();
            let rgba_buffer: Vec<[u8, ..4]> = bitmap.buffer().as_slice().iter().map(|&a| {
                [0, 0, 0, a]
            }).collect();
            self.graphics.device.update_texture(&texture, &image_info,
                                                &rgba_buffer).unwrap();
            let sampler = self.graphics.device.create_sampler(
                gfx::tex::SamplerInfo::new(gfx::tex::Bilinear,
                                           gfx::tex::Clamp)
                    );

            let params = TextShaderParams {
                tex: (texture, Some(sampler)),
                color: [1.0, 0.0, 0.0, 1.0],
            };

            let x2 = x + (g.bitmap_left() as f32) * sx;
            let y2 = -y - (g.bitmap_top() as f32) * sy;
            let w = (bitmap.width() as f32) * sx;
            let h = (bitmap.rows() as f32) * sy;

            let verts = vec![
                TextVertex { coord: [x2, -y2, 0.0, 0.0] },
                TextVertex { coord: [x2 + w, -y2, 1.0, 0.0] },
                TextVertex { coord: [x2, -y2 - h, 0.0, 1.0] },
                TextVertex { coord: [x2 + w, -y2 - h, 1.0, 1.0] },
                ];
            let mesh = self.graphics.device.create_mesh(verts);
            let batch = self.graphics.make_batch(&mesh, mesh.get_slice(gfx::TriangleStrip),
                                                 &self.text_program, &self.state).unwrap();
            self.graphics.draw(&batch, &params, &self.frame);

            x += ((g.advance().x >> 6) as f32) * sx;
            y += ((g.advance().y >> 6) as f32) * sy;
        }
    }
}
