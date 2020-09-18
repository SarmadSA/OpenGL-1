#version 430 core

in vec3 position;
in layout(location=1) vec4 inColors; //Get color matrix as input
out layout(location=1) vec4 outColors; //Output color matrix to fragment shader

uniform layout(location=3) mat4 matrix; //

void main()
{
    gl_Position = vec4(position, 1.0f) * matrix; //Transform 
    outColors = inColors;
}