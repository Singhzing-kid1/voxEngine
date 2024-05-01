#include "main.hpp"

float deltaTime = 0.0f;	// time between current frame and last frame
float lastFrame = 0.0f;

int main(int argc, char* argv[]){
    SDL_Window* window;
    SDL_GLContext context;

    if(SDL_Init(SDL_INIT_VIDEO) < 0){
        cout << "could not initialize SDL. Error: " << SDL_GetError();
        return 1;
    }

    window = SDL_CreateWindow("v0.0.3", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, HEIGHT, WIDTH, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);

    if(window == nullptr){
        cout << "SDL could not create window. Error: " << SDL_GetError();
        return 1;
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);

    context = SDL_GL_CreateContext(window);

    if(context == nullptr){
        cout << "SDL could not create a openGL context. Error: " << SDL_GetError();
        return 1;
    }

    SDL_GL_SetSwapInterval(0);

    glewExperimental = GL_TRUE;
    GLenum glewError = glewInit();

    if(glewError != GLEW_OK){
        cout << "Could not initialize GLEW. Error: " << glewGetErrorString(glewError);
        return 1;
    }

    SDL_Event e;
    bool quit = false;

    glEnable(GL_DEPTH_TEST);

    Shader testShader("./shaders/vertex.glsl", "./shaders/frag.glsl");
    Camera mainCam(45.0f, (float)HEIGHT, (float)WIDTH, 0.1f, 100.0f, vec3(0.0f, 0.0f, 3.0f));
    Model testModel("./models/testModel.model", 0.03125f);

    mat4 model = mat4(1.0f);

    mat4 view = mat4(1.0f);
    view = translate(view, mainCam.getPosVec());

    mat4 projection;
    projection = perspective(mainCam.getFov(), mainCam.getAspect(), mainCam.getNear(), mainCam.getFar());

    float cameraSpeed = 0.05f;

    float accumX = 0.0f, accumY = 0.0f;
    float lastX = 0.0, lastY = 0.0;
    int mouseX, mouseY;
    float xOffset = 0.0, yOffset = 0.0;
    float sensitivity = 0.5;
    float yaw = 0.0f, pitch = 0.0f;

    SDL_SetWindowGrab(window, SDL_TRUE);

    while(!quit){
        float currentFrame = SDL_GetTicks()/1000.0f;
        deltaTime = currentFrame - lastFrame;
        lastFrame = currentFrame;

        xOffset = accumX - lastX;
        yOffset = lastY - accumY;

        lastX = accumX;
        lastY = accumY;

        xOffset *= sensitivity;
        yOffset *= sensitivity;

        yaw += xOffset;
        pitch += yOffset;

        if(pitch > 89.0f){
            pitch = 89.0f;
        }
        if(pitch < -89.0f){
            pitch = -89.0f;
        }

        mainCam.setYaw(yaw);
        mainCam.setPitch(pitch);

        while(SDL_PollEvent(&e)){
            cameraSpeed = 2.5 * deltaTime;
            if(e.type == SDL_QUIT){
                quit = true;
            }

            if(e.type == SDL_KEYDOWN){

                if(e.key.keysym.sym == SDLK_ESCAPE){
                    quit = true;
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_W){
                    mainCam.moveCamera("ws", cameraSpeed);
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_S){
                    mainCam.moveCamera("ws", -cameraSpeed);
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_D){
                    mainCam.moveCamera("ad", cameraSpeed);
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_A){
                    mainCam.moveCamera("ad", -cameraSpeed);
                }

            }

            if(e.type == SDL_MOUSEMOTION){
                accumX -= abs(e.motion.xrel) > 1 ? e.motion.xrel : 0;
                accumY += abs(e.motion.yrel) > 1 ? e.motion.yrel : 0;
                cout << "xRel: " << e.motion.xrel << ", yRel: " << e.motion.yrel << endl;

                SDL_GetMouseState(&mouseX, &mouseY);

                float newMouseX = mouseX + e.motion.xrel;
                float newMouseY = mouseY + e.motion.yrel;

                if(newMouseX < WIDTH/2 || newMouseX > WIDTH/2 || newMouseY < HEIGHT/2 || newMouseY > HEIGHT/2){
                    SDL_WarpMouseInWindow(window, HEIGHT/2, WIDTH/2);
                }
            }
        }
        mainCam.update();

        view = lookAt(mainCam.getPosVec(), mainCam.getFrontVec() + mainCam.getPosVec(), mainCam.getUpVec());

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        testModel.render(testShader, model, view, projection);

        SDL_Delay(5);
        SDL_GL_SwapWindow(window);
    }

    return 1;
}