#version 430 core

layout(location=1) in vec4 inColors;
out vec4 color;

void main()
{
    color = vec4(inColors);
}