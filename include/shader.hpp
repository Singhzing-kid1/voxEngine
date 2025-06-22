#ifndef SHADER_HPP
#define SHADER_HPP

#include "main.hpp"

class Shader{
    public:
        /**
         * @brief Construct a new Shader object
         * 
         * THIS OVERLOAD IS FOR CLASSES THAT INHERIT FROM THIS CLASS
         * 
         */
        Shader();

        /**
         * @brief Construct a new Shader object
         * 
         * @param vertexPath path to the vertex shader file
         * @param fragmentPath path to the fragment shader file
         * 
         */
        Shader(const char*, const char*);
        
        /**
         * @brief use the current shader program
         * 
         */
        void use();

        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data mat4
         */
        void setUniform(std::string, mat4);
        
        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data vec2
         * 
         */
        void setUniform(std::string, vec2);

        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data vec3
         * 
         */
        void setUniform(std::string, vec3);

        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data int
         * 
         */
        void setUniform(std::string, int);

        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data float
         * 
         */
        void setUniform(std::string, float);

        /**
         * @brief Set the Uniform object
         * 
         * @param name name of the uniform
         * @param data bool
         * 
         */
        void setUniform(std::string, bool);

    protected:
        unsigned int ID;

        /**
         * @brief read the shader code
         * 
         * @param path path to the shader source code
         * 
         * @return std::string 
         */
        std::string readCode(const char*);
};

#endif
