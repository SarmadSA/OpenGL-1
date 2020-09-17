extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()



// == // Modify and complete the function below for the first task
//This function sets up a vertex array object (VAO), it takes two arguments that is the data (vertices) and the indices (used to fill the index buffer to tell which vertices should be connected together) and it returns the VAO ID
unsafe fn setup_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>) -> u32 { 
    let mut array: u32 = 0; //a pointer to a location where the generated VAO ID can be stored. since we only are allocating a single VAO I created this empty unsigned int
    gl::GenVertexArrays(1, &mut array); //This will generate a VAO
    gl::BindVertexArray(array); //This will bind the VAO

    let mut bufferID: u32 = 0; //Here the ID of the buffer (VBO) will be stored
    gl::GenBuffers(1, &mut bufferID); // This will genereate a buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, bufferID);// this will bind the buffer in the created earlier in last line

    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&vertices), pointer_to_array(&vertices), gl::STATIC_DRAW); //Here we will fill the buffer with our data
    

    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null()); //Here we define a format for our buffer (because we didnt tell OpenGL about our data, so it does not know if we passed x,y or x,y,z etc, here we tell it)
    gl::EnableVertexAttribArray(0); //This will enable the pointer. index is same as in previwes line


    //INDEX BUFFER (index buffer spesifies how the vertecies in databuffer should be combined together, else we wont know which points are connected and not)
    let mut bufferID2: u32 = 0; //Soter the ID of the buffer
    gl::GenBuffers(1, &mut bufferID2); //Generate buffer
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bufferID2);// bind the buffer

    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(&indices), pointer_to_array(&indices), gl::STATIC_DRAW);//Fill wil indices


    let mut bufferID3: u32 = 0; 
    gl::GenBuffers(1, &mut bufferID3); //Generate buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, bufferID3);// bind the buffer //TODO ARRAY_BUFFER? or other?

    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&colors), pointer_to_array(&colors), gl::STATIC_DRAW);//Fill with colors //TODO ARRAY_BUFFER? or other?

    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null()); //Here we define a format for our buffer (because we didnt tell OpenGL about our data, so it does not know if we passed x,y or x,y,z etc, here we tell it)
    gl::EnableVertexAttribArray(1); //This will enable the pointer. index is same as in previwes line

    //Find the max (for index (the first parameter) in function glVertexattribPointer)
    //int maxVertexAttribs;
    //glGetIntegerv(gl::MAX_VERTEX_ATTRIBS, &maxVertexAttribs);
    //printf("gl::MAX_VERTEX_ATTRIBS: %i\n", maxVertexAttribs);

    array //At the end we return the VAO ID. We will use this ID to refer to the array whenever we want to do something with it
} 

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);
    
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO here

    //Here I setup the VAO. As mentioned earlier this returns the array ID which I have to use later to draw the primitive
let value = unsafe {
    let vertices: Vec<f32> = vec![
        //Triangle 
        -0.4, -0.3, 0.6,
        0.1, -0.3, 0.6,
       -0.2,  0.3, 0.6,
       -0.2, -0.3, 0.7,
        0.4, -0.3, 0.7,
        0.2,  0.3, 0.7,
       -0.3, -0.6, 0.8,
        0.3, -0.6, 0.8,
        0.0, -0.0, 0.8,
    ];

    let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let colors: Vec<f32> = vec![
        0.0, 0.0, 1.0, 0.4,
        0.0, 0.0, 1.0, 0.4,
        0.0, 0.0, 1.0, 0.4,
        1.0, 0.0, 0.0, 0.4,
        1.0, 0.0, 0.0, 0.4,
        1.0, 0.0, 0.0, 0.4,
        0.0, 1.0, 0.0, 0.4,
        0.0, 1.0, 0.0, 0.4,
        0.0, 1.0, 0.0, 0.4,
        ];
    setup_vao(&vertices, &indices, &colors)    
};


/*     
        let value = unsafe {
    let vertices: Vec<f32> = vec![
        //Triangle 
        -0.40, -0.25, 0.6,
        0.10, -0.25, 0.6,
       -0.15,  0.30, 0.6,
       -0.10, -0.25, 0.7,
        0.40, -0.25, 0.7,
        0.15,  0.30, 0.7,
       -0.25, -0.58, 0.8,
        0.25, -0.58, 0.8,
        0.00, -0.03, 0.8,
    ];

        let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let colors: Vec<f32> = vec![
            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,
            1.0, 0.0, 0.0, 0.4,
            1.0, 0.0, 0.0, 0.4,
            1.0, 0.0, 0.0, 0.4,
            0.0, 1.0, 0.0, 0.4,
            0.0, 1.0, 0.0, 0.4,
            0.0, 1.0, 0.0, 0.4,
            ];
        setup_vao(&vertices, &indices, &colors)    
    };
    */

    // Basic usage of shader helper
    // The code below returns a shader object, which contains the field .program_id
    // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path)

    //Here I load the shaders, the vertex shader and the fragment shader then they are linked.
    let shader = unsafe{
        shader::ShaderBuilder::new().attach_file("./shaders/simple.vert").attach_file("./shaders/simple.frag").link()
    };

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        },
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        },


                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here

                //Here I am using the program ID I get retuned from the shader object
                gl::UseProgram(shader.program_id);

                
                let scaling: glm::Mat4 = glm::scaling(&glm::vec3(1.0,1.0,1.0));
                let identity: glm::Mat4 = glm::identity(); 
                let translation: glm::Mat4 = glm::translation(&glm::vec3(0.0,0.0,-5.0));
                let projection: glm::Mat4 = glm::perspective(0.75, 0.75, 1.0, 100.0);

                let transformationCombo: glm::Mat4 = projection * translation;

                gl::UniformMatrix4fv(3, 1, gl::FALSE, transformationCombo.as_ptr());

                gl::DrawElements(gl::TRIANGLES, 9, gl::UNSIGNED_INT, ptr::null());

            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    
                    Space => {
                        print!("Space pressed!!!");

                    },

                    D => {
                        print!("D pressed!!!");
                    },
                    
                    KEY_D => {
                        print!("Some other key pressed!!!");
                    },

                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}