/**
 * @file shaders.hpp
 * @author Veer Singh
 * @brief Makes, compiling, building, sending data to/from, and using GLSL shaders.
 * @version 0.0.3
 * @date 2024-07-27
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#ifndef SHADER_HPP
#define SHADER_HPP

#include "main.hpp"

class Shader{
    public:
        unsigned int ID;

        Shader(const char*, const char*);

        void use();

        void setUniform(string, mat4);
        void setUniform(string, vec2);
        void setUniform(string, vec3);
        void setUniform(string, int);
        void setUniform(string, float);
        void setUniform(string, bool);
};
#endif