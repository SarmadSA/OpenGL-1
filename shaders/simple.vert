#version 430 core

in vec3 position;
in layout(location=1) vec4 inColors; //Get color matrix as input
out layout(location=1) vec4 outColors; //Output color matrix to fragment shader

uniform layout(location=3) mat4 matrix; //uniform matrix passed in from our main loop at location 3. Contains the transformation matrix.

void main()
{
    gl_Position = vec4(position, 1.0f) * matrix; //Transform 
    outColors = inColors; //Assign out colors the value of the in colors to be passed to the fragment shader
}