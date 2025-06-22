#include "main.hpp"

Compute::Compute(const char* path) : Shader(){
    std::string codeStr = readCode(path);

    const char* _code = codeStr.c_str();

    unsigned int code;
    int success;
    char infoLog[512];

    code = glCreateShader(GL_COMPUTE_SHADER);
    glShaderSource(code, 1, &_code, nullptr);
    glCompileShader(code);
    glGetShaderiv(code, GL_COMPILE_STATUS, &success);
    if(!success){
        glGetShaderInfoLog(code, 512, nullptr, infoLog);
        cout << "shader could not compile: " << infoLog << "\n";
    }

    ID = glCreateProgram();
    glAttachShader(ID, code);
    glLinkProgram(ID);
    glGetProgramiv(ID, GL_LINK_STATUS, &success);
    if(!success){
        glGetProgramInfoLog(ID, 512, nullptr, infoLog);
        cout << "shader program could not link: " << infoLog << "\n";
    }

    glDeleteShader(code);
}