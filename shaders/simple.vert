#version 430 core

in vec3 position;
in layout(location=1) vec4 inColors; //Get color matrix as input
out layout(location=1) vec4 outColors; //Output color matrix to fragment shader

in layout(location=2) vec3 inNormals; //Get normals as input
out layout(location=2) vec3 outNormals; //Output normals to the fragment shader

uniform layout(location=3) mat4 matrix; //uniform matrix passed in from our main loop at location 3. Contains the transformation matrix.
uniform layout(location=4) mat4 modelMatrix; //This is the model matrix passed from draw_scene fucntion in main.rs
uniform layout(location=5) mat4 viewProjectionMatrix; //This is the view projection matrix passed from draw_scene fucntion in main.rs


void main()
{
    gl_Position = vec4(position, 1.0f) * matrix; //Transform 
    outColors = inColors; //Assign out colors the value of the in colors to be passed to the fragment shader
    outNormals = normalize(mat3(modelMatrix) * inNormals);; //Assign out normals the value of the in normals to be passed to the fragment shader
}