/**
 * @file engine.cpp
 * @author Veer Singh
 * @brief implementation of the engine class. handles window creation and input handling.
 * @version 0.1
 * @date 2024-09-02
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#include "main.hpp"

Engine::Engine(int height, int width, const char* title){
    this->width = width;
    this->height = height;

    if(SDL_Init(SDL_INIT_VIDEO) < 0){
        cout << "could not init SDL2. Error: " << SDL_GetError();
    }

    window = SDL_CreateWindow(title, SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, width, height, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);

    if(window == nullptr){
        cout << "could not create Window. Error: " << SDL_GetError();
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 2);

    context = SDL_GL_CreateContext(window);

    if(context == nullptr){
        cout << "could not create openGL context. Error: " << SDL_GetError();
    }

    SDL_GL_SetSwapInterval(0);

    glewExperimental = GL_TRUE;
    GLenum glewError = glewInit();

    if(glewError != GLEW_OK){
        cout << "could not init GLEW. Error: " << glewGetErrorString(glewError);
    }

    glEnable(GL_DEPTH_TEST);
}


void Engine::eventHandling(inputData* data){
    data->state = SDL_GetKeyboardState(NULL);
    float currentFrame = SDL_GetTicks()/1000.0f;
    deltaTime = currentFrame - lastFrame;
    lastFrame = currentFrame;

    data->xOffset = accumX - lastX;
    data->yOffset = lastY - accumY;

    lastX = accumX;
    lastY = accumY;

    while(SDL_PollEvent(&e)){
        switch(e.type){
            case SDL_QUIT:
                data->shouldQuit = false;
                break;

            case SDL_KEYDOWN:
                if(e.key.keysym.sym == SDLK_ESCAPE){
                    data->shouldQuit = true;
                }
                break;

            case SDL_MOUSEMOTION:
                accumX -= e.motion.xrel;
                accumY += e.motion.yrel;

                SDL_GetMouseState(&mouseX, &mouseY);

                float newMouseX = mouseX + e.motion.xrel;
                float newMouseY = mouseY + e.motion.yrel;
                
                if(newMouseX < width/2 || newMouseX > width/2 || newMouseY < height/2 || newMouseY > height/2){
                    SDL_WarpMouseInWindow(window, height/2, width/2);
                } 

        }
    }

}

void Engine::initRendering(vec3 pos, float fov, float aspect, float nearPlane, float farPlane){
    SDL_SetRelativeMouseMode(SDL_TRUE);

    view = translate(view, pos);
    projection = perspective(fov, aspect, nearPlane, farPlane);
}

void Engine::swap(){
    SDL_GL_SwapWindow(window);
}