#version 430 core

layout(location=1) in vec4 inColors; //Get the color from location 1 (output from vertex shader)
layout(location=2) in vec3 inNormals; //Get the normals from location 2 (output from vertex shader)
out vec4 color;

vec3 lightDirection =  normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    float maxi = max(0, dot(inNormals , (-lightDirection)));
    color = vec4(inColors[0] * maxi, inColors[1] * maxi, inColors[2] * maxi, inColors[3]);//Color the geometry
    //color = inColors * vec4(max(vec3(0),inNormals * (-lightDirection)), 1.0);//Use color matrix to color the geometry
}