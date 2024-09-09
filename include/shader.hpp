#ifndef SHADER_HPP
#define SHADER_HPP

#include "main.hpp"

class Shader{
    public:
        Shader(const char*, const char*);

        unsigned int ID;
        
        void use();

        void setUniform(std::string, mat4);
        void setUniform(std::string, vec2);
        void setUniform(std::string, vec3);
        void setUniform(std::string, int);
        void setUniform(std::string, float);
        void setUniform(std::string, bool);
    
};

#endif
