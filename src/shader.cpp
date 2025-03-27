#include "main.hpp"

Shader::Shader(const char* vertexPath, const char* fragmentPath){
    std::string vertString;
    std::string fragString;
    ifstream vertex;
    ifstream fragment;

    vertex.exceptions(ifstream::failbit | ifstream::badbit);
    fragment.exceptions(ifstream::failbit | ifstream::badbit);

    try{
        vertex.open(vertexPath);
        fragment.open(fragmentPath);
        stringstream vertexStream, fragmentStream;

        vertexStream << vertex.rdbuf();
        fragmentStream << fragment.rdbuf();

        vertex.close();
        fragment.close();

        vertString = vertexStream.str();
        fragString = fragmentStream.str();
    } catch(ifstream::failure e){
        cout << "could not read shaders\n";
    }

    const char* vertCode = vertString.c_str();
    const char* fragCode = fragString.c_str();

    unsigned int vert, frag;
    int success;
    char infoLog[512];

    vert = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vert, 1, &vertCode, NULL);
    glCompileShader(vert);
    glGetShaderiv(vert, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(vert, 512, NULL, infoLog);
        cout << "vertex could not compile: " << infoLog << "\n";
    }

    frag = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(frag, 1, &fragCode, NULL);
    glCompileShader(frag);
    glGetShaderiv(frag, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(frag, 512, NULL, infoLog);
        cout << "fragment could not compile: " << infoLog << "\n";
    }

    ID = glCreateProgram();
    glAttachShader(ID, vert);
    glAttachShader(ID, frag);
    glLinkProgram(ID);
    glGetProgramiv(ID, GL_LINK_STATUS, &success);
    if(!success){
        glGetProgramInfoLog(ID, 512, NULL, infoLog);
        cout << "shader program could not link: " << infoLog << "\n";
    }

    glDeleteShader(vert);
    glDeleteShader(frag);


}

void Shader::use(){
    glUseProgram(ID);
}

void Shader::setUniform(std::string name, mat4 data){
    glUniformMatrix4fv(glGetUniformLocation(ID, name.c_str()), 1, GL_FALSE, value_ptr(data));
}

void Shader::setUniform(std::string name, vec3 data){
    glUniform3f(glGetUniformLocation(ID, name.c_str()), data.x, data.y, data.z);
}

void Shader::setUniform(std::string name, vec2 data){
    glUniform2f(glGetUniformLocation(ID, name.c_str()), data.x, data.y);
}

void Shader::setUniform(std::string name, int data){
    glUniform1i(glGetUniformLocation(ID, name.c_str()), data);
}

void Shader::setUniform(std::string name, float data){
    glUniform1f(glGetUniformLocation(ID, name.c_str()), data);
}

void Shader::setUniform(std::string name, bool data){
    glUniform1i(glGetUniformLocation(ID, name.c_str()), (int)data);
}