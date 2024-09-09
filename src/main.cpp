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

    Player mainPlayer(45, main.height, main.width, 0.1f, 1000.0f, vec3(0.0f, 5.0f, 0.0f), vec4(1.0f, 1.0f, 3.0f, 1.0f), 2.5f);
    World world(16, 32);
    Shader shader("./shaders/vertex.glsl", "./shaders/fragment.glsl");

    main.initRendering(mainPlayer.getItem(1), mainPlayer.getItem(1.0f), mainPlayer.getItem(2.0f), mainPlayer.getItem(3.0f), mainPlayer.getItem(4.0f));

    inputData data;
    bool quit = false;

    while(!quit){
        data = main.eventHandling();
        mainPlayer.updatePlayer(main.deltaTime, data.state, data.xOffset, data.yOffset);

        main.view = lookAt(mainPlayer.getItem(1), mainPlayer.getItem(3) + mainPlayer.getItem(1), mainPlayer.getItem(2));
        world.update();

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        world.render(main.model, main.view, main.projection, shader);

        main.swap();
        quit = data.shouldQuit;
    }
    return 0;
}