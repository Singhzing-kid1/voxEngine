#include "main.hpp"

namespace debug{
    GLuint createTextureFromSurface(SDL_Surface* surface){
        GLuint texID;
        glGenTextures(1, &texID);
        glBindTexture(GL_TEXTURE_2D, texID);

        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

        GLint maxTexSize = 0;
        glGetIntegerv(GL_MAX_TEXTURE_SIZE, &maxTexSize);

        GLenum texFormat = GL_RGBA;
        GLint internalFormat = GL_RGBA;
        int bytesPerPixel = surface->format->BytesPerPixel;
        switch (bytesPerPixel){
            case 4:
                internalFormat = GL_RGBA;
                if(surface->format->Rmask == 0x000000ff){
                    pass;
                } else {
                    texFormat = GL_BGRA;
                }
                break;

            case 3:
                internalFormat = GL_RGB;
                if(surface->format->Rmask == 0x000000ff){
                    texFormat = GL_RGB;
                } else {
                    texFormat = GL_BGR;
                }
                break;
        }

        glTexImage2D(GL_TEXTURE_2D, 0, internalFormat, surface->w, surface->h, 0, texFormat, GL_UNSIGNED_BYTE, surface->pixels);
        
        return texID;
    }

    void renderTextGL(TTF_Font* font, const char* text, SDL_Color color, float x, float y){
        if(!font) cout << "font :(  " << TTF_GetError() << "\n";
        SDL_Surface* surface = TTF_RenderUTF8_Blended(font, text, color);
        if(!surface) return;

        cout << "in func" << text << "\n";

        GLuint texture = createTextureFromSurface(surface);

        

        glEnable(GL_TEXTURE_2D);
        glBindTexture(GL_TEXTURE_2D, texture);

        glBegin(GL_QUADS);
            glTexCoord2f(0, 0); glVertex2f(x, y);
            glTexCoord2f(1, 0); glVertex2f(x + (float)surface->w, y);
            glTexCoord2f(1, 1); glVertex2f(x + (float)surface->w, y + (float)surface->h);
            glTexCoord2f(0, 1); glVertex2f(x, y + (float)surface->h);
        glEnd();

        glDisable(GL_TEXTURE_2D);
        
        glDeleteTextures(1, &texture);
        SDL_FreeSurface(surface);
    }
}