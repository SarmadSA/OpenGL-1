#version 430 core

in vec3 position;
in layout(location=1) vec4 inColors;
out layout(location=1) vec4 outColors;

mat4 matrix; //Create a metrix with 1's in the leading diagonal

void main()
{
    //Creating an identity matrix:

    //Method 1: 
    //matrix = mat4(1);

    //Method 2:
    //matrix[0][0] = 1.0;
    //matrix[1][1] = 1.0;
    //matrix[2][2] = 1.0;
    //matrix[3][3] = 1.0;

    //Method 3:
    matrix[0] = vec4(1,0,0,0);
    matrix[1] = vec4(0,1,0,0);
    matrix[2] = vec4(0,0,1,0);
    matrix[3] = vec4(0,0,0,1);

    gl_Position = vec4(position, 1.0f) * matrix;
    outColors = inColors;
}