#include "main.hpp"

float deltaTime = 0.0f;	// time between current frame and last frame
float lastFrame = 0.0f;

int main(){
    SDL_Window* window;
    SDL_GLContext context;

    if(SDL_Init(SDL_INIT_VIDEO) < 0){
        cout << "could not initialize SDL. Error: " << SDL_GetError();
        return 1;
    }

    window = SDL_CreateWindow("v0.0.1", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 640, 480, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);

    if(window == nullptr){
        cout << "SDL could not create window. Error: " << SDL_GetError();
        return 1;
    }

    #ifdef _WIN32
    SDL_SysWMinfo info;
    SDL_VERSION(&info.version);
    if (!SDL_GetWindowWMInfo(window, &info)) {
        cout << "cant get window WM info: " << SDL_GetError();
        return 1;
    }

    HWND hwnd = info.info.win.window;
    #endif

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

    Shader testShader("../shaders/vertex.glsl", "../shaders/frag.glsl");


    float vertices[] = {
        0.5f, 0.5f, 0.5f, 
        0.5f, -0.5f, 0.5f, 
        -0.5f, 0.5f, 0.5f, 
        -0.5f, -0.5f, 0.5f,
        0.5f, 0.5f, -0.5f, 
        0.5f, -0.5f, -0.5f, 
        -0.5f, 0.5f, -0.5f, 
        -0.5f, -0.5f, -0.5f
    };

    unsigned int indices[] = {
        2, 0, 1,
        2, 3, 1,
        3, 1, 5,
        3, 7, 5,
        7, 5, 4,
        7, 6, 4,
        6, 4, 0,
        6, 2, 0,
        1, 5, 4,
        1, 0, 4,
        2, 3, 7,
        2, 6, 7
    };

    unsigned int VBO, EBO, VAO;
    glGenVertexArrays(1, &VAO);
    glGenBuffers(1, &VBO);
    glGenBuffers(1, &EBO);

    glBindVertexArray(VAO);

    glBindBuffer(GL_ARRAY_BUFFER, VBO);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, EBO);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);

    glBindVertexArray(0);

    glEnable(GL_DEPTH_TEST);

    mat4 model = mat4(1.0f);
    model = rotate(model, radians(30.0f), vec3(1.0f, 0.0f, 0.0f));

    mat4 view = mat4(1.0f);
    view = translate(view, vec3(0.0f, 0.0f, -3.0f));

    mat4 projection;
    projection = perspective(radians(45.0f), 800.0f/600.0f, 0.1f, 100.0f);

    vec3 cameraPos = vec3(0.0f, 0.0f, 3.0f);
    vec3 cameraFront = vec3(0.0f, 0.0f, -1.0f);
    vec3 cameraUp = vec3(0.0f, 1.0f, 0.0f);

    float cameraSpeed = 0.05f;

    glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);

    while(!quit){
        float currentFrame = SDL_GetTicks()/1000.0f;
        deltaTime = currentFrame - lastFrame;
        cout << deltaTime << endl;
        lastFrame = currentFrame;
        
        while(SDL_PollEvent(&e)){
            cameraSpeed = 2.5f * deltaTime;
            if(e.type == SDL_QUIT){
                quit = true;
            }

            if(e.type == SDL_KEYDOWN){
                if(e.key.keysym.sym == SDLK_w){
                    cameraPos += cameraSpeed * cameraFront;
                }

                if(e.key.keysym.sym == SDLK_s){
                    cameraPos -= cameraSpeed * cameraFront;
                }

                if(e.key.keysym.sym == SDLK_d){
                    cameraPos += cameraSpeed * normalize(cross(cameraFront, cameraUp));
                }

                if(e.key.keysym.sym == SDLK_a){
                    cameraPos -= cameraSpeed * normalize(cross(cameraFront, cameraUp));
                }
            }
        }

        view = lookAt(cameraPos, cameraPos + cameraFront, cameraUp);

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        testShader.setUniform("model", model);
        testShader.setUniform("view", view);
        testShader.setUniform("projection", projection);

        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
        glEnableVertexAttribArray(0);

        testShader.use();
        glBindVertexArray(VAO);
        glDrawElements(GL_TRIANGLES, 36, GL_UNSIGNED_INT, 0);

        SDL_Delay(20);
        SDL_GL_SwapWindow(window);
    }
}