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

int main(int argc, char* argv[]){
    Engine main(1080, 1920, "v0.0.6");

    Player mainPlayer(45, main.height, main.width, 0.1f, 1000.0f, vec3(0.0f, 1.0f, 0.0f), vec4(1.0f, 1.0f, 3.0f, 1.0f), 2.5f);
    Shader shader("./shaders/vertex.glsl", "./shaders/fragment.glsl");
    World world(1.0f, 20, 20, 21);
    main.initRendering(mainPlayer.getItem(1), mainPlayer.getItem(1.0f), mainPlayer.getItem(2.0f), mainPlayer.getItem(3.0f), mainPlayer.getItem(4.0f));

    glEnable(GL_CULL_FACE);
    glCullFace(GL_BACK);
    glFrontFace(GL_CCW);

    inputData data;
    bool quit = false;

    float pitch = 0.0f , yaw = 90.0f;

    while(!quit){

        main.eventHandling(&data);

        pitch += data.yOffset;
        yaw += data.xOffset;

        if(pitch > 89.0f){
            pitch = 89.0f;
        }
        if(pitch < -89.0f){
            pitch = -89.0f;
        }

        mainPlayer.updatePlayer(main.deltaTime, data.state, yaw, pitch);

        main.view = lookAt(mainPlayer.getItem(1), mainPlayer.getItem(3) + mainPlayer.getItem(1), mainPlayer.getItem(2));

        world.update(mainPlayer.position, main.lastFrame);

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        world.render(main.model, main.view, main.projection, shader, mainPlayer.position);

        main.swap();
        quit = data.shouldQuit;
    }

    return 0;
}