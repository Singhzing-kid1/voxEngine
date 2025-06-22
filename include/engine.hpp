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
        /**
         * @brief Construct a new Engine object
         * 
         * @param height height of window
         * @param width width of window
         * @param title title of window
         * 
         */
        Engine(int, int, const char*);
        float deltaTime, lastFrame;

        /**
         * @brief handles input events
         * 
         * @param data pointer to an instance of the inputData struct
         * 
         */
        void eventHandling(inputData*);

        /**
         * @brief creates the view and projection matricies
         * 
         * @param pos position of player
         * @param fov field of view
         * @param aspect aspect ratio of window
         * @param nearPlane near plane
         * @param farPlane far plane
         * 
         */
        void initRendering(vec3, float, float, float, float);
        
        /**
         * @brief updates the SDL2 window with the openGL framebuffer
         * 
         */
        void swap();

        mat4 model = mat4(1.0f), view = mat4(1.0f), projection;

        int width, height;
        btCollisionWorld* collisionWorld;

    private:
        SDL_Event e;

        SDL_Window* window;
        SDL_GLContext context;

        double accumX = 0.0, accumY = 0.0;
        int mouseX = 0, mouseY = 0;
        float lastX = 0.0f, lastY = 0.0f;
};



#endif