#include "main.hpp"

Shader::Shader(const char* vertexPath, const char* fragmentPath){
    std::string vertString = readCode(vertexPath);
    std::string fragString = readCode(fragmentPath);

    const char* vertCode = vertString.c_str();
    const char* fragCode = fragString.c_str();

    unsigned int vert, frag;
    int success;
    char infoLog[512];

    vert = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vert, 1, &vertCode, nullptr);
    glCompileShader(vert);
    glGetShaderiv(vert, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(vert, 512, nullptr, infoLog);
        cout << "vertex could not compile: " << infoLog << "\n";
    }

    frag = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(frag, 1, &fragCode, nullptr);
    glCompileShader(frag);
    glGetShaderiv(frag, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(frag, 512, nullptr, infoLog);
        cout << "fragment could not compile: " << infoLog << "\n";
    }

    ID = glCreateProgram();
    glAttachShader(ID, vert);
    glAttachShader(ID, frag);
    glLinkProgram(ID);
    glGetProgramiv(ID, GL_LINK_STATUS, &success);
    if(!success){
        glGetProgramInfoLog(ID, 512, nullptr, infoLog);
        cout << "shader program could not link: " << infoLog << "\n";
    }

    glDeleteShader(vert);
    glDeleteShader(frag);


}

Shader::Shader(){}

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

std::string Shader::readCode(const char* path){
    std::string shader;
    ifstream code;

    code.exceptions(ifstream::failbit | ifstream::badbit);

    try{
        code.open(path);
        stringstream shaderStream;

        shaderStream << code.rdbuf();

        code.close();

        shader = shaderStream.str();
    } catch(ifstream::failure e){
        cout << "could not read shader source\n";
    }

    return shader;
}