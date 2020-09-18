#version 430 core

layout(location=1) in vec4 inColors; //Get the color from location 1 (output from vertex shader)
out vec4 color;

void main()
{
    color = vec4(inColors); //Use color matrix to color the geometry
}