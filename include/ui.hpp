#ifndef UI_HPP
#define UI_HPP

#include "main.hpp"

class Shader;

class UI{
    public:
        UI(int, int);


        enum ElementType {TEXT, BUTTON, BORDER};

        int width, height;

        void render(Shader);

        int addElement(ElementType, std::string, TTF_Font*, SDL_Color, int, int);
        int addElement(ElementType, int, int);

        void editElement(int, vec2, TTF_Font*, SDL_Color, std::string);
        void update();

    protected:
        

        struct uiElement{
            int ID;
            vec2 pos;
            SDL_Surface* surface;
            ElementType type;
            GLuint vao, vbo, ebo; // init when element created
            vector<vec2> verts, texs;
            vector<unsigned int> inds;
        };

        mat4 projection;

        vector<uiElement> elements;



        void genMesh();
        void updateBuffers();
};

#endif