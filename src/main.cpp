/**
 * @file main.cpp
 * @author Veer Singh
 * @brief The Magic Happens here
 * @version 0.0.6
 * @date 2024-09-02
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#include "main.hpp"


#ifdef _WIN32
extern "C" {  //temporary to force computer to use Nvidia GPU or AMD GPU over integrated graphics
    __declspec(dllexport) unsigned long NvOptimusEnablement = 0x00000001;
    __declspec(dllexport) int AmdPowerXpressRequestHighPerformance = 1;
}
#endif

int main(int argc, char* argv[]){
    Engine main(1080, 1920, "v0.0.8 pre-release");

    SDL_Color color = {255, 255, 255, 255};

    Player mainPlayer(45, main.height, main.width, 0.1f, 1000.0f, vec3(0.0f, 0.3125f, 0.0f), vec4(1.0f, 1.0f, 3.0f, 1.0f), 2.5f);
    Shader shader("./shaders/vertex.glsl", "./shaders/fragment.glsl");
    Shader textShader("./shaders/textVert.glsl", "./shaders/textFrag.glsl");
    World world(0.03125f, 2000, 20, 51, "asdkjfhsadkfjhekjlahsdlkjdfheljkshadf21230984322"); // g7Kp1zQw8vR3xJt5LmSd2Xy9BnHa4UcEoTfS
    UI debug(main.width, main.height);
    int deltaTimeUI = debug.addElement(UI::ElementType::TEXT, " ", TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, 0, 0);
    int fpsUI = debug.addElement(UI::ElementType::TEXT, " ", TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, 0, 25);
    int rendererUI = debug.addElement(UI::ElementType::TEXT, (std::string)reinterpret_cast<const char*>(glGetString(GL_RENDERER)), TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, 0, 50);
    int versionUI = debug.addElement(UI::ElementType::TEXT, (std::string)reinterpret_cast<const char*>(glGetString(GL_VERSION)), TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, 0, 75);
    main.initRendering(mainPlayer.getItem(1), mainPlayer.getItem(1.0f), mainPlayer.getItem(2.0f), mainPlayer.getItem(3.0f), mainPlayer.getItem(4.0f));
    const GLubyte* renderer = glGetString(GL_RENDERER);

    glEnable(GL_CULL_FACE);
    glCullFace(GL_BACK);
    glFrontFace(GL_CCW);

    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA); 

    //glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);

    inputData data;
    bool quit = false;

    float pitch = 0.0f , yaw = 0.0f;

    while(!quit){
        main.eventHandling(&data);
        world.sync(mainPlayer.position);

        pitch += data.yOffset;
        yaw += data.xOffset;

        if(pitch > 89.0f){
            pitch = 89.0f;
        }
        if(pitch < -89.0f){
            pitch = -89.0f;
        }

        mainPlayer.updatePlayer(main.deltaTime, data.state, yaw, pitch, main.collisionWorld);

        main.view = lookAt(mainPlayer.getItem(1), mainPlayer.getItem(3) + mainPlayer.getItem(1), mainPlayer.getItem(2));

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        world.update();
        debug.update();
        
        world.render(main.model, main.view, main.projection, shader, mainPlayer.position);
        debug.render(textShader);

        stringstream buffer;
        buffer << main.deltaTime * 1000.0f << " ms";
        debug.editElement(deltaTimeUI, vec2(0, 0), TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, buffer.str());

        buffer.str(std::string());
        buffer << (1 / main.deltaTime) << "fps";
        debug.editElement(fpsUI, vec2(0, 25), TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15), color, buffer.str());   

        main.swap();

        quit = data.shouldQuit;
    }

    return 0;
}