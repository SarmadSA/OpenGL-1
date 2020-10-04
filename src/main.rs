extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
mod toolbox;

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
unsafe fn setup_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals: &Vec<f32>) -> u32 { 
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


    //Color
    let mut bufferID3: u32 = 0; 
    gl::GenBuffers(1, &mut bufferID3); //Generate buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, bufferID3);// bind the buffer
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&colors), pointer_to_array(&colors), gl::STATIC_DRAW);//Fill with colors
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null()); //Here we define a format for our buffer (because we didnt tell OpenGL about our data, so it does not know if we passed x,y or x,y,z etc, here we tell it)
    gl::EnableVertexAttribArray(1); //This will enable the pointer. index is same as in previous line


    //Normal
    let mut bufferID4: u32 = 0; 
    gl::GenBuffers(1, &mut bufferID4); //Generate buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, bufferID4);// bind the buffer
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(&normals), pointer_to_array(&normals), gl::STATIC_DRAW);//Fill with normals
    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, ptr::null()); //Here we define a format for our buffer (because we didnt tell OpenGL about our data, so it does not know if we passed x,y or x,y,z etc, here we tell it)
    gl::EnableVertexAttribArray(2); //This will enable the pointer. index is same as in previous line

    //Find the max (for index (the first parameter) in function glVertexattribPointer)
    //int maxVertexAttribs;
    //glGetIntegerv(gl::MAX_VERTEX_ATTRIBS, &maxVertexAttribs);
    //printf("gl::MAX_VERTEX_ATTRIBS: %i\n", maxVertexAttribs);

    array //At the end we return the VAO ID. We will use this ID to refer to the array whenever we want to do something with it
} 

unsafe fn draw_scene(root: &scene_graph::SceneNode, view_projection_matrix: &glm::Mat4){

    //MVP matirx
    let mvp_matrix: glm::Mat4 = root.current_transformation_matrix * view_projection_matrix;

    //Check if node is drawable, set uniforms, draw
    if(root.index_count > -1){
        gl::UniformMatrix4fv(4, 1, gl::FALSE, root.current_transformation_matrix.as_ptr());//Pass the model matrix as a uniform variable to the vertex shader. this will be used to transfrom the vertex normal and fix the lighting as the helicopter turns
        gl::UniformMatrix4fv(5, 1, gl::FALSE, view_projection_matrix.as_ptr());//Pass the view projection matrix as a uniform variable to the vertex shader at location 5, used to transform the input vertext

        gl::UniformMatrix4fv(3, 1, gl::FALSE, mvp_matrix.as_ptr());

        gl::BindVertexArray(root.vao_id); //bind
        gl::DrawElements(gl::TRIANGLES, root.index_count, gl::UNSIGNED_INT, ptr::null()); //Draw
    }

    for &child in &root.children {
        draw_scene(&*child, &view_projection_matrix);
    }
}


unsafe fn update_node_transformations(root: &mut scene_graph::SceneNode, transformation_so_far: &glm::Mat4){
    
    //construct the correct transformation matrix
    let translationMatrix: glm::Mat4 = glm::translation(&root.position); //This is the translation matrix
    let transposeTranslation: glm::Mat4 = glm::transpose(&translationMatrix); //Transpose the translation matrix

    let rotationX: glm::Mat4 = glm::rotation(root.rotation[0], &glm::vec3(root.reference_point[0],0.0,0.0)); //Rotate about the x-axis 
    let rotationY: glm::Mat4 = glm::rotation(root.rotation[1], &glm::vec3(0.0,root.reference_point[1],0.0)); //Rotate about the y-axis 
    let rotationZ: glm::Mat4 = glm::rotation(root.rotation[2], &glm::vec3(0.0,0.0,root.reference_point[2])); //Rotate about the z-axis 
    let rotationMatrix: glm::Mat4 = rotationX * rotationY * rotationZ; //Rotate about the z-axis 
    
    //Move the root to the origin
    //Translate by the inverse of a reference point

    let transformationMatrix: glm::Mat4 = rotationMatrix * transposeTranslation; // here I should multiply by the translation matrix, however, I have a problem with it, it causes a wierd effect (I think shearing) and I was not able to solve it in time
    let translateee: glm::Mat4 = glm::translation(&root.reference_point);

    //update the node's transformation matrix
    root.current_transformation_matrix = transformationMatrix * transformation_so_far;
    
    for &child in &root.children {
        update_node_transformations(&mut *child, &root.current_transformation_matrix);
     }
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

    //Here I load the lunarsurface.obj
    let terrain_mesh = mesh::Terrain::load("./resources/lunarsurface.obj"); //Load the lunar surface

    //Here I setup the VAO. As mentioned earlier this returns the array ID which I have to use later to draw the primitive     
    let value = unsafe {
        let vertices: Vec<f32> = terrain_mesh.vertices;
        let indices: Vec<u32> = terrain_mesh.indices;
        let colors: Vec<f32> = terrain_mesh.colors;
        let normals: Vec<f32> = terrain_mesh.normals;
        setup_vao(&vertices, &indices, &colors, &normals)    
    };


    let heli_mesh = mesh::Helicopter::load("./resources/helicopter.obj"); //Load helicopter

    let heli_body_vao = unsafe {
        let vertices: Vec<f32> = heli_mesh.body.vertices;
        let indices: Vec<u32> = heli_mesh.body.indices;
        let colors: Vec<f32> = heli_mesh.body.colors;
        let normals: Vec<f32> = heli_mesh.body.normals;
        setup_vao(&vertices, &indices, &colors, &normals)    
    };


    let heli_main_rotor_vao = unsafe {
        let vertices: Vec<f32> = heli_mesh.main_rotor.vertices;
        let indices: Vec<u32> = heli_mesh.main_rotor.indices;
        let colors: Vec<f32> = heli_mesh.main_rotor.colors;
        let normals: Vec<f32> = heli_mesh.main_rotor.normals;
        setup_vao(&vertices, &indices, &colors, &normals)    
    };

    let heli_tail_rotor_vao = unsafe {
        let vertices: Vec<f32> = heli_mesh.tail_rotor.vertices;
        let indices: Vec<u32> = heli_mesh.tail_rotor.indices;
        let colors: Vec<f32> = heli_mesh.tail_rotor.colors;
        let normals: Vec<f32> = heli_mesh.tail_rotor.normals;
        setup_vao(&vertices, &indices, &colors, &normals)    
    };


    let heli_door_vao = unsafe {
        let vertices: Vec<f32> = heli_mesh.door.vertices;
        let indices: Vec<u32> = heli_mesh.door.indices;
        let colors: Vec<f32> = heli_mesh.door.colors;
        let normals: Vec<f32> = heli_mesh.door.normals;
        setup_vao(&vertices, &indices, &colors, &normals)    
    };
    
    // Basic usage of shader helper
    // The code below returns a shader object, which contains the field .program_id
    // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path)

    //Here I load the shaders, the vertex shader and the fragment shader then they are linked.
    let shader = unsafe{
        shader::ShaderBuilder::new().attach_file("./shaders/simple.vert").attach_file("./shaders/simple.frag").link()
    };

    //Here I create a scene graph
    let mut root_scene_node = scene_graph::SceneNode::new();//Generate a root scene node
    let mut terrain_scene_node = scene_graph::SceneNode::from_vao(value, terrain_mesh.index_count);//Generate a scene node for the terrain
    
    let mut heli_body_node = scene_graph::SceneNode::from_vao(heli_body_vao, heli_mesh.body.index_count);//Generate a scene node for the helicopter body
    let mut heli_main_rotor_node = scene_graph::SceneNode::from_vao(heli_main_rotor_vao, heli_mesh.main_rotor.index_count);//Generate a scene node for the helicopter body
    let mut heli_tail_rotor_node = scene_graph::SceneNode::from_vao(heli_tail_rotor_vao, heli_mesh.tail_rotor.index_count);//Generate a scene node for the helicopter body
    let mut heli_door_node = scene_graph::SceneNode::from_vao(heli_door_vao, heli_mesh.door.index_count);//Generate a scene node for the helicopter body

    //Initilize the values in the scene node data structure to initial values
    heli_tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);
    heli_body_node.reference_point = glm::vec3(-0.68, -0.19, -4.13);
    heli_body_node.position = glm::vec3(0.5, 0.5, 0.0); //Set position of helicopter body for testing
    heli_body_node.rotation = glm::vec3(0.0, 3.0, 0.0); //Set rotation of helicopter body for testing

    heli_body_node.add_child(&heli_main_rotor_node); //Add main rotor as a child node to helicopter
    heli_body_node.add_child(&heli_tail_rotor_node); //Add tail rotor as a child node to helicopter
    heli_body_node.add_child(&heli_door_node); //Add door as a child node to helicopter
    terrain_scene_node.add_child(&heli_body_node); //Add helicopter body as a child node to terrain node
    root_scene_node.add_child(&terrain_scene_node); //Add terrain scene node to the root node

    //Here I debug
    root_scene_node.print();
    terrain_scene_node.print();
    heli_body_node.print();

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        //Create needed variables for translation (camera movement)
        let mut _x = 0.0;
        let mut _y = 0.0;
        let mut _z = -3.0;

        //Create needed variables for rotations (camera rotations)
        let mut rot_x = 0.0;
        let mut rot_y = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        let identity: glm::Mat4 = glm::identity(); //Create identitiy matrix
        let projection: glm::Mat4 = glm::perspective(1.00, 1.00, 1.0, 1000.0); //Projection

        let mut angel = 0.0;

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;
            
            let speed = 1.0;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {

                         /*Use WASDEQ for camera movements*/
                        VirtualKeyCode::W => {
                            _z += speed;
                        },
                        VirtualKeyCode::S => {
                            _z -= speed;
                        },
                        VirtualKeyCode::E => {
                            _y += speed;
                        },
                        VirtualKeyCode::Q => {
                            _y -= speed;
                        },
                        VirtualKeyCode::A => {
                            _x += speed;
                        },
                        VirtualKeyCode::D => {
                            _x -= speed;
                        },

                        /* Use arrows for camera rotations*/ 
                        VirtualKeyCode::Down => {
                            rot_x -= delta_time;
                        },
                        VirtualKeyCode::Up => {
                            rot_x += delta_time;
                        },
                        VirtualKeyCode::Right => {
                            rot_y -= delta_time;
                        },
                        VirtualKeyCode::Left => {
                            rot_y += delta_time;
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


                //let scaling: glm::Mat4 = glm::scaling(&glm::vec3(1.0,1.0,1.0));

                //Translation
                let translation: glm::Mat4 = glm::translation(&glm::vec3(_x,_y,_z)); //Translate, this gives us the camera movements (upward, downward, sideways (left and right), forward and backward)
                let transposeTranslation: glm::Mat4 = glm::transpose(&translation); //Transpose the translation matrix

                //Rotation
                let rotationX: glm::Mat4 = glm::rotation(rot_x, &glm::vec3(1.0,0.0,0.0)); //Rotate about the x-axis 
                let rotationY: glm::Mat4 = glm::rotation(rot_y, &glm::vec3(0.0,1.0,0.0)); //Rotate about the y-axis
                let transposeRotationX: glm::Mat4 = glm::transpose(&rotationX); //Transpose rotationX matrix
                let transposeRotationY: glm::Mat4 = glm::transpose(&rotationY); //Transpose rotationY matrix 

                //Produce the tranformation matrics from individual transformations                
                let transformationCombo: glm::Mat4 = transposeRotationX * transposeRotationY * transposeTranslation *  projection * identity; //Multiply to get the transformation matrix which is then passed to the vertex shader to apply the transformation
                
                heli_tail_rotor_node.rotation = glm::vec3(angel, 0.0, 0.0);
                //heli_main_rotor_node.rotation = glm::vec3(0.0, 0.0, angel);
                angel +=1.0 * elapsed;
                
                let headding = toolbox::simple_heading_animation(elapsed);
                heli_body_node.rotation = glm::vec3(headding.pitch, headding.yaw, headding.roll);
                heli_body_node.position = glm::vec3(headding.x, 0.0, headding.z);

                update_node_transformations(&mut root_scene_node, &glm::identity());
                draw_scene(&root_scene_node , &transformationCombo);
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