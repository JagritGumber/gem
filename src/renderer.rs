use crate::display::GemDisplay;
use glutin::display::GetGlDisplay;
use glutin::prelude::GlDisplay;
use std::ffi::CString;

pub struct GemRenderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
}

impl GemRenderer {
    pub fn new(display: &GemDisplay) -> Self {
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            display
                .gl_context
                .display()
                .get_proc_address(symbol.as_c_str())
                .cast()
        });

        println!("[GemRenderer] OpenGL loaded");

        let version = unsafe {
            let data = gl::GetString(gl::VERSION) as *const i8;
            std::ffi::CStr::from_ptr(data).to_str().unwrap().to_string()
        };
        println!("[GemRenderer] OpenGL version: {}", version);

        let program = unsafe { Self::create_shader_program() };

        let (vao, vbo, ebo) = unsafe { Self::create_quad_buffers() };

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.1, 0.1, 0.15, 1.0);
        }

        Self {
            program,
            vao,
            vbo,
            ebo,
        }
    }

    unsafe fn create_shader_program() -> gl::types::GLuint {
        let vertex_src = r#"
            #version 330 core
            layout (location = 0) in vec2 aPos;
            layout (location = 1) in vec2 aTexCoord;
            layout (location = 2) in vec4 aColor;
            
            out vec2 TexCoord;
            out vec4 Color;
            
            uniform mat4 projection;
            
            void main() {
                gl_Position = projection * vec4(aPos, 0.0, 1.0);
                TexCoord = aTexCoord;
                Color = aColor;
            }
        "#;

        let fragment_src = r#"
            #version 330 core
            in vec2 TexCoord;
            in vec4 Color;
            out vec4 FragColor;
            
            uniform sampler2D texture1;
            uniform bool useTexture;
            
            void main() {
                if (useTexture) {
                    FragColor = texture(texture1, TexCoord) * Color;
                } else {
                    FragColor = Color;
                }
            }
        "#;

        let vertex_shader = unsafe { Self::compile_shader(vertex_src, gl::VERTEX_SHADER) };
        let fragment_shader = unsafe { Self::compile_shader(fragment_src, gl::FRAGMENT_SHADER) };

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
        }

        // Check for linking errors
        let mut success = 0;
        unsafe {
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            let mut len = 0;
            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            }
            let mut buffer = vec![0u8; len as usize];
            unsafe {
                gl::GetProgramInfoLog(program, len, &mut len, buffer.as_mut_ptr() as *mut i8);
            }
            panic!(
                "Program linking failed: {}",
                String::from_utf8_lossy(&buffer)
            );
        }

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        println!("[GemRenderer] Shader program created");

        program
    }

    unsafe fn compile_shader(src: &str, shader_type: gl::types::GLenum) -> gl::types::GLuint {
        let shader = unsafe { gl::CreateShader(shader_type) };
        let c_str = CString::new(src.as_bytes()).unwrap();
        unsafe {
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
        }

        // Check for compilation errors
        let mut success = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            let mut len = 0;
            unsafe {
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            }
            let mut buffer = vec![0u8; len as usize];
            unsafe {
                gl::GetShaderInfoLog(shader, len, &mut len, buffer.as_mut_ptr() as *mut i8);
            }
            panic!(
                "Shader compilation failed: {}",
                String::from_utf8_lossy(&buffer)
            );
        }

        shader
    }

    unsafe fn create_quad_buffers() -> (gl::types::GLuint, gl::types::GLuint, gl::types::GLuint) {
        // Vertex data: position (x, y), texcoord (u, v), color (r, g, b, a)
        #[rustfmt::skip]
        let vertices: [f32; 32] = [
            // positions   // texcoords  // colors
            -0.5, -0.5,    0.0, 0.0,     1.0, 1.0, 1.0, 1.0,  // bottom-left
             0.5, -0.5,    1.0, 0.0,     1.0, 1.0, 1.0, 1.0,  // bottom-right
             0.5,  0.5,    1.0, 1.0,     1.0, 1.0, 1.0, 1.0,  // top-right
            -0.5,  0.5,    0.0, 1.0,     1.0, 1.0, 1.0, 1.0,  // top-left
        ];

        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = 8 * std::mem::size_of::<f32>() as i32;

            // Position attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // TexCoord attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Color attribute
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (4 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);
        }

        println!(
            "[GemRenderer] Quad buffers created (VAO: {}, VBO: {}, EBO: {})",
            vao, vbo, ebo
        );

        (vao, vbo, ebo)
    }

    pub fn begin_frame(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn render_quad(&self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        unsafe {
            gl::UseProgram(self.program);

            let mut model = [0.0f32; 16];
            model[0] = width;
            model[5] = height;
            model[10] = 1.0;
            model[12] = x;
            model[13] = y;
            model[15] = 1.0;

            let projection = Self::ortho_matrix(-1.0, 1.0, -1.0, 1.0);
            let mvp = Self::multiply_matrices(&projection, &model);

            let proj_loc =
                gl::GetUniformLocation(self.program, CString::new("projection").unwrap().as_ptr());
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, mvp.as_ptr());

            let use_texture_loc =
                gl::GetUniformLocation(self.program, CString::new("useTexture").unwrap().as_ptr());
            gl::Uniform1i(use_texture_loc, 0);

            #[rustfmt::skip]
            let vertices: [f32; 32] = [
                // positions   // texcoords  // colors
                -0.5,  0.5,    0.0, 1.0,     color[0], color[1], color[2], color[3],
                 0.5,  0.5,    1.0, 1.0,     color[0], color[1], color[2], color[3],
                 0.5, -0.5,    1.0, 0.0,     color[0], color[1], color[2], color[3],
                -0.5, -0.5,    0.0, 0.0,     color[0], color[1], color[2], color[3],
            ];

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
            );

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            gl::BindVertexArray(0);
        }
    }

    fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
        let mut matrix = [0.0f32; 16];
        matrix[0] = 2.0 / (right - left);
        matrix[5] = 2.0 / (top - bottom);
        matrix[10] = -1.0;
        matrix[12] = -(right + left) / (right - left);
        matrix[13] = -(top + bottom) / (top - bottom);
        matrix[15] = 1.0;
        matrix
    }

    fn multiply_matrices(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
        let mut result = [0.0f32; 16];
        for i in 0..4 {
            for j in 0..4 {
                result[i * 4 + j] = a[i * 4 + 0] * b[0 * 4 + j]
                    + a[i * 4 + 1] * b[1 * 4 + j]
                    + a[i * 4 + 2] * b[2 * 4 + j]
                    + a[i * 4 + 3] * b[3 * 4 + j];
            }
        }
        result
    }

    pub fn set_viewport(&self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }
}

impl Drop for GemRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
