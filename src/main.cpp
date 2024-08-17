#include "main.hpp"

float deltaTime = 0.0f;	// time between current frame and last frame
float lastFrame = 0.0f;

bool checkIfVec3InVolume(vec3 point, vec3 camera,  float volumeHalfSideLength){
    bool x = abs(point.x - camera.x) <= volumeHalfSideLength ? true : false;
    bool y = abs(point.y - camera.y) <= volumeHalfSideLength ? true : false;
    bool z = abs(point.z - camera.z) <= volumeHalfSideLength ? true : false;

    return x && y && z;
}


int main(int argc, char* argv[]){
    SDL_Window* window;
    SDL_GLContext context;

    if(SDL_Init(SDL_INIT_VIDEO) < 0){
        cout << "could not initialize SDL. Error: " << SDL_GetError();
        return 1;
    }

    window = SDL_CreateWindow("v0.0.5", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, HEIGHT, WIDTH, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);

    if(window == nullptr){
        cout << "SDL could not create window. Error: " << SDL_GetError();
        return 1;
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 1);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 2);

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
    glLineWidth(1.0f);


    btBroadphaseInterface* broadPhase = new btDbvtBroadphase();
    btDefaultCollisionConfiguration* config = new btDefaultCollisionConfiguration();
    btCollisionDispatcher* dispatch = new btCollisionDispatcher(config);

    btCollisionWorld* collisionWorld = new btCollisionWorld(dispatch, broadPhase, config);

    Shader testShader("./shaders/vertexWithSizeScaling.glsl", "./shaders/frag.glsl");
    Shader noScalingShader("./shaders/vertexWithOutSizeScaling.glsl", "./shaders/frag.glsl");
    // Camera mainCam(45.0f, (float)HEIGHT, (float)WIDTH, 0.1f, 10000.0f, vec3(0.0f, 0.0f, 3.0f));
    Model chunkZeroBorder("./models/chunkBorder.model", 0.5f, vec3(0, 0, 0), vec3(0, 0, 0), false, "cZB", false, collisionWorld);
    Model testModel("./models/testModel.model", 1.0f, vec3(0, 0, 0), vec3(0, 0, 0), true, "tM", true, collisionWorld);
    Model points("./models/pointsOfChunk.model", .025f, vec3(0, 0, 0), vec3(0, 0, 0), false, "p", false, collisionWorld);

    Player player(45.0f, (float)HEIGHT, (float)WIDTH, 0.1f, 10000.0f, vec3(0.0f, 0.0f, 12.0f), vec3(2.0f, 4.0f, 2.0f));

    vector<Model> models;

    models.push_back(points);
    models.push_back(testModel);
    models.push_back(chunkZeroBorder);

    mat4 model = mat4(1.0f);

    mat4 view = mat4(1.0f);
    view = translate(view, player.getPosVec());

    mat4 projection;
    projection = perspective(player.getFov(), player.getAspect(), player.getNear(), player.getFar());

    bool camCollided = false;

    float cameraSpeed = 0.75f;

    double accumX = 0.0, accumY = 0.0;
    int mouseX, mouseY;
    float xOffset = 0.0f, yOffset = 0.0f, lastX, lastY;
    float sensitivity = 0.25;
    float yaw = 0.0f, pitch = 0.0f;

    int renderDistance = 12; // in chunks
    
    SDL_SetRelativeMouseMode(SDL_TRUE);
    //glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);

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

        player.setYaw(yaw);
        player.setPitch(pitch); 
        player.setDeltaTime(deltaTime);

        //mainCam.smoothUpdate(yaw, pitch, deltaTime);

        const  Uint8* state = SDL_GetKeyboardState(NULL);
        
        if(state[SDL_SCANCODE_W] && state[SDL_SCANCODE_D]){
            player.movePlayer(Camera::NORTHEAST, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_W] && state[SDL_SCANCODE_A]){
            player.movePlayer(Camera::NORTHWEST, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_S] && state[SDL_SCANCODE_D]){
            player.movePlayer(Camera::SOUTHEAST, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_S] && state[SDL_SCANCODE_A]){
            player.movePlayer(Camera::SOUTHWEST, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_W] && !state[SDL_SCANCODE_S]){
            player.movePlayer(Camera::NORTH, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_S] && !state[SDL_SCANCODE_W]){
            player.movePlayer(Camera::SOUTH, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_D] && !state[SDL_SCANCODE_A]){
            player.movePlayer(Camera::EAST, cameraSpeed, collisionWorld);
        } else if(state[SDL_SCANCODE_A] && !state[SDL_SCANCODE_D]){
            player.movePlayer(Camera::WEST, cameraSpeed, collisionWorld);
        }

        while(SDL_PollEvent(&e)){
            cameraSpeed = 2.5 * deltaTime;
            if(e.type == SDL_QUIT){
                quit = true;
            }

            if(e.type == SDL_KEYDOWN){

                if(e.key.keysym.sym == SDLK_ESCAPE){
                    quit = true;
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_M){
                    player.setPos(vec3(0.0f, 0.0f, 17.0f));
                }

                if(e.key.keysym.scancode == SDL_SCANCODE_B){
                    player.setPos(vec3(0.0f, 17.0f, 0.0f));
                }

                
                if(e.key.keysym.scancode == SDL_SCANCODE_N){
                    player.setPos(vec3(0.0f, 0.0f, -17.0f));
                }
            }

            if(e.type == SDL_MOUSEMOTION){
                accumX -= e.motion.xrel;
                accumY += e.motion.yrel;

                SDL_GetMouseState(&mouseX, &mouseY);

                float newMouseX = mouseX + e.motion.xrel;
                float newMouseY = mouseY + e.motion.yrel;
                
                if(newMouseX < WIDTH/2 || newMouseX > WIDTH/2 || newMouseY < HEIGHT/2 || newMouseY > HEIGHT/2){
                    SDL_WarpMouseInWindow(window, HEIGHT/2, WIDTH/2);
                } 
            }
        }
        player.pUpdate(models, collisionWorld);

        view = lookAt(player.getPosVec(), player.getFrontVec() + player.getPosVec(), player.getUpVec());

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        for(auto &m : models){
            if(checkIfVec3InVolume(m.getChunkCoord(), floor((player.getPlayerPos() + vec3(17.0f)) / 34.0f), renderDistance)){
                m.getShouldScalePositionBool() ? m.render(testShader, model, view, projection) : m.render(noScalingShader, model, view, projection);
            }

            m.getId() == "p" ? m.setChunkCoord(floor((player.getPlayerPos() + vec3(17.0f)) / 34.0f)) : (void)0;
        }

        SDL_GL_SwapWindow(window);
    }

    return 1;
}