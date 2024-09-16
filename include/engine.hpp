/**
 * @file engine.hpp
 * @author Veer Singh
 * @brief definition of the engine class. handles window creation and input handling.
 * @version 0.0.6
 * @date 2024-09-02
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#ifndef ENGINE_HPP
#define ENGINE_HPP

#include "main.hpp"

typedef struct{
    float xOffset = 0.0f, yOffset = 0.0f;
    bool shouldQuit = false;
    const Uint8* state;
} inputData;

class Engine{

    public:
        Engine(int, int, const char*);
        float deltaTime, lastFrame;
        void eventHandling(inputData*);

        void initRendering(vec3, float, float, float, float);

        void swap();

        mat4 model = mat4(1.0f), view = mat4(1.0f), projection;

        int width, height;

    private:
        SDL_Event e;

        SDL_Window* window;
        SDL_GLContext context;

        double accumX = 0.0, accumY = 0.0;
        int mouseX, mouseY;
        float lastX, lastY;
};



#endif