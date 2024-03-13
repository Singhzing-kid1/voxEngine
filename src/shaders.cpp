#include "main.hpp"

Shader::Shader(const char* vertexPath, const char* fragPath){
    string vertexCodeStr;
    string fragCodeStr;
    ifstream vShaderFile;
    ifstream fShaderFile;

    vShaderFile.exceptions(ifstream::failbit | ifstream::badbit);
    fShaderFile.exceptions(ifstream::failbit | ifstream::badbit);

    try{
        vShaderFile.open(vertexPath);
        fShaderFile.open(fragPath);
        stringstream vShaderStream, fShaderStream;

        vShaderStream << vShaderFile.rdbuf();
        fShaderStream << fShaderFile.rdbuf();

        vShaderFile.close();
        fShaderFile.close();

        vertexCodeStr = vShaderStream.str();
        fragCodeStr = fShaderStream.str();
    } catch(ifstream::failure e){
        cout << "could not read shaders" << endl;
    }

    const char* vertexCode = vertexCodeStr.c_str();
    const char* fragCode = fragCodeStr.c_str();

    unsigned int vertex, frag;
    int success;
    char infoLog[512];

    vertex = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex, 1, &vertexCode, NULL);
    glCompileShader(vertex);
    glGetShaderiv(vertex, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(vertex, 512, NULL, infoLog);
        cout << "vertex shader could not compile. Error log: " << infoLog;
    }

    frag = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(frag, 1, &fragCode, NULL);
    glCompileShader(frag);
    glGetShaderiv(frag, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(frag, 512, NULL, infoLog);
        cout << "frag shader could not compile. Error log: " << infoLog;
    }

    ID = glCreateProgram();
    glAttachShader(ID, vertex);
    glAttachShader(ID, frag);
    glLinkProgram(ID);
    glGetProgramiv(ID, GL_LINK_STATUS, &success);
    if(!success){
        glGetProgramInfoLog(ID, 512, NULL, infoLog);
        cout << "shader program could not link. Error log: " << infoLog;    
    }

    glDeleteShader(vertex);
    glDeleteShader(frag);
}

void Shader::use(){
    glUseProgram(ID);
}

void Shader::setUniform(string name, mat4 data){
    glUniformMatrix4fv(glGetUniformLocation(ID, name.c_str()), 1, GL_FALSE, value_ptr(data));
}

void Shader::setUniform(string name, vec3 data){
    glUniform3f(glGetUniformLocation(ID, name.c_str()), data.x, data.y, data.z);
}

void Shader::setUniform(string name, vec2 data){
    glUniform2f(glGetUniformLocation(ID, name.c_str()), data.x, data.y);
}

void Shader::setUniform(string name, int data){
    glUniform1i(glGetUniformLocation(ID, name.c_str()), data);
}

void Shader::setUniform(string name, float data){
    glUniform1f(glGetUniformLocation(ID, name.c_str()), data);
}

void Shader::setUniform(string name, bool data){
    glUniform1i(glGetUniformLocation(ID, name.c_str()), (int)data);
        }
