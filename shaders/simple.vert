#version 430 core

in vec3 position;
in layout(location=1) vec4 inColors;
out layout(location=1) vec4 outColors;

void main()
{
    gl_Position = vec4(position, 1.0f);
    outColors = inColors;
}