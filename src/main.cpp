/**
 * @file main.cpp
 * @author Veer Singh
 * @brief The Magic Happens here
 * @version 0.0.8
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
    Engine engine(1080, 1920, "v0.0.8 pre-release"); // init engine class
    SDL_Color color = {255, 255, 255, 255}; // white color for UI text

    Player mainPlayer(45, engine.height, engine.width, 0.1f, 1000.0f, vec3(0.0f, 5.0f, 0.0f), vec4(1.0f, 1.0f, 3.0f, 1.0f), 2.5f); // create player

    Shader shader("./shaders/vertex.vert", "./shaders/fragment.frag");  // terrain shader
    Shader textShader("./shaders/textVert.vert", "./shaders/textFrag.frag"); // ui shader

    Compute computeShader("./shaders/testComp.comp"); // test compute shader

    World world(100, 64, 7, "asdkjfhsadkfjhekjlahsdlkjdfheljkshadf21230984322", mainPlayer.position.xz()); // g7Kp1zQw8vR3xJt5LmSd2Xy9BnHa4UcEoTfS | world init

    UI debug(engine.width, engine.height, TTF_OpenFont("./fonts/IBMPlexMono-Regular.ttf", 15)); // create debug UI
    int deltaTimeUI = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 0); // add element for deltaTime display
    int deltaTimeWorldUpdate = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 25);
    int deltaTimeDebugUpdate = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 50);
    int deltaTimeWorldDraw = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 75);
    int deltaTimeDebugDraw = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 100);
    int playerPitchAndYaw = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 125);
    int fpsUI = debug.addElement(UI::ElementType::TEXT, " ", color, 0, 150); // add element for FPS display
    int rendererUI = debug.addElement(UI::ElementType::TEXT, (std::string)reinterpret_cast<const char*>(glGetString(GL_RENDERER)), color, 0, 175); // add element to show current renderer(GPU)
    int versionUI = debug.addElement(UI::ElementType::TEXT, (std::string)reinterpret_cast<const char*>(glGetString(GL_VERSION)), color, 0, 200); // add element to show OpenGL version and graphics driver version 

    engine.initRendering(mainPlayer.getItem(1), mainPlayer.getItem(1.0f), mainPlayer.getItem(2.0f), mainPlayer.getItem(3.0f), mainPlayer.getItem(4.0f)); // initialize rendering

    glEnable(GL_CULL_FACE); // enable backface culling and specify direction of front face
    glCullFace(GL_BACK);
    glFrontFace(GL_CCW);

    glEnable(GL_BLEND); // enable blending allows UI to render with transparent frags
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA); 

    //glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);

    inputData data;
    bool quit = false;
    bool showDebug = false;

    float pitch = 0.0f , yaw = 0.0f;
    while(!quit){
        engine.eventHandling(&data); // input handling

        // calculate pitch and yaw based on mouse movement
        pitch += data.yOffset;
        yaw += data.xOffset;

        if(pitch > 89.0f){
            pitch = 89.0f;
        }
        if(pitch < -89.0f){
            pitch = -89.0f;
        }

        // update the player position/orientation
        mainPlayer.updatePlayer(engine.deltaTime, data.state, yaw, pitch, engine.collisionWorld);

        // update the view matrix based on player orientaion
        engine.view = lookAt(mainPlayer.getItem(1), mainPlayer.getItem(3) + mainPlayer.getItem(1), mainPlayer.getItem(2));
        
        // clear the screen
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        float worldUStart = (SDL_GetTicks64()/1000.0f);
        world.update(mainPlayer.position, SDL_GetTicks64()/1000.0f); // update the world (creates mesh)
        world.sendBuffers();
        float wUpdateTime = (SDL_GetTicks64()/1000.0f) - worldUStart;
        
        float debugUStart = (SDL_GetTicks64()/1000.0f);
        debug.update(); // updaye the debug UI (creats UI meshes)
        float dUpdateTime = (SDL_GetTicks64()/1000.0f) - debugUStart; 
        
        float worldRStart = (SDL_GetTicks64()/1000.0f);
        world.render(engine.model, engine.view, engine.projection, mainPlayer.position, shader); // render the world
        float wRenderTime = (SDL_GetTicks64()/1000.0f) - worldRStart;

        if(showDebug){
            float debugRStart = (SDL_GetTicks64()/1000.0f);
            debug.render(textShader); // render UI
            float dRenderTime = (SDL_GetTicks64()/1000.0f) - debugRStart; 

            // update the deltaTime UI element with the current deltaTime
            stringstream buffer;
            buffer << floor(engine.deltaTime * 1000.0f) << " ms";
            debug.editElement(deltaTimeUI, vec2(0, 0), color, buffer.str()); 

            buffer.str(std::string());
            buffer << "world update: " << floor(wUpdateTime * 1000.0f) << "ms";
            debug.editElement(deltaTimeWorldUpdate, vec2(0, 25), color, buffer.str());   

            buffer.str(std::string());
            buffer << "debug update: " << floor(dUpdateTime * 1000.0f) << "ms";
            debug.editElement(deltaTimeDebugUpdate, vec2(0, 50), color, buffer.str());   

            buffer.str(std::string());
            buffer << "world draw: " << floor(wRenderTime * 1000.0f) << "ms";
            debug.editElement(deltaTimeWorldDraw, vec2(0, 75), color, buffer.str());   

            buffer.str(std::string());
            buffer << "debug draw: " << floor(dRenderTime * 1000.0f) << "ms";
            debug.editElement(deltaTimeDebugDraw, vec2(0, 100), color, buffer.str());  
            
            buffer.str(std::string());
            buffer << "Yaw: " << yaw << ", Pitch: " << pitch << ", Pos: " << to_string(mainPlayer.position);
            debug.editElement(playerPitchAndYaw, vec2(0, 125), color, buffer.str());

            // update the fps UI element with the current fps 
            buffer.str(std::string());
            buffer << floor(1 / engine.deltaTime) << "fps";
            debug.editElement(fpsUI, vec2(0, 150), color, buffer.str());
        }   

        engine.swap(); // swap framebuffers

        quit = data.shouldQuit; // check if reached a quit condition
    }

    debug.~UI();
    world.~World();
    engine.~Engine();

    return 0;
}