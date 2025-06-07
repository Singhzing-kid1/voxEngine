#ifndef DEBUG_HPP
#define DEBUG_HPP

#include "main.hpp"

namespace debug{
    GLuint createTextureFromSurface(SDL_Surface*);
    void renderTextGL(TTF_Font*, const char*, SDL_Color, float, float);
}

#endif