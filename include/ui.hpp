#ifndef UI_HPP
#define UI_HPP

#include "main.hpp"

class Shader;

class UI{
    public:
        /**
         * @brief Construct a new UI object
         * 
         * @param width width of screen
         * @param height height of screen
         */
        UI(int, int);

        enum ElementType {TEXT, BUTTON, BORDER};

        int width, height;
        
        /**
         * @brief renders the ui elements
         * 
         * @param shader shader program used to render the text
         */
        void render(Shader);

        /**
         * @brief adds an element to the elements vector and returns the element id so it can be edited
         * 
         * @param type type of ui element
         * @param text text to be rendered
         * @param font font of the text(TrueTypeFont file)
         * @param color color of the text
         * @param x x position of text
         * @param y y position of text
         * 
         * @return int
         */
        int addElement(ElementType, std::string, TTF_Font*, SDL_Color, int, int);
        int addElement(ElementType, int, int);

        /**
         * @brief edit an element that already exists
         * 
         * @param id the id of the element to be edited
         * @param pos position of the edited element
         * @param font font of the text(TrueTypeFont file)
         * @param color color of the text
         * @param text text to be rendered
         * 
         */
        void editElement(int, vec2, TTF_Font*, SDL_Color, std::string);

        /**
         * @brief update the mesh and buffers
         * 
         */
        void update();

    private:
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

        /**
         * @brief generate the vertex, index, and tex coords for each ui element
         * 
         */
        void genMesh();

        /**
         * @brief create the openGL buffers to send vertex, index and texture coordinate data to gpu
         * 
         */
        void updateBuffers();
};

#endif