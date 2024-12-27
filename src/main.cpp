/**
 * @file main.cpp
 * @author Veer Singh
 * @brief main game loop stuff
 * @version 0.0.7
 * @date 2024-12-01
 * 
 * @copyright Copyright (c) 2024
 * 
 */

#include "main.hpp"

int main(int argc, char* argv[]){
    Engine engine(1920, 1080, "voxEngine 0.0.7");
    InputData data;

    while(true){
        engine.eventHandling(&data);

        if(data.shouldQuit){
            break;
        }
    }

    engine.~Engine();
}