#ifndef COMPUTE_HPP
#define COMPUTE_HPP

#include "main.hpp"

class Shader;

class Compute : public Shader {
    public:
        /**
         * @brief Construct a new Compute object
         * 
         * inherits from Shader()
         * 
         * @param path path to the compute shader source
         */
        Compute(const char *);
};

#endif