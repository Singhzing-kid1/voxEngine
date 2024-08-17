/**
 * @file player.hpp
 * @author Veer Singh
 * @brief Manage collisions and general physics of a controllable player
 * @version 0.0.5
 * @date 2024-08-05
 * 
 * @copyright Copyright (c) 2024
 * 
 */

#ifndef PLAYER_HPP
#define PLAYER_HPP

#include "main.hpp"

class Camera;
class Model;

class Player : public Camera {
    public:
        Player(float, float, float, float, float, vec3, vec3);

        void pUpdate(vector<Model>, btCollisionWorld* collisionWorld);

        vec3 getPlayerPos();
        vec3 getPlayerVelocity();

        void movePlayer(Axis, float, btCollisionWorld*);

        void setDeltaTime(float);

    private:
        vec3 position, size, velocity, prevPos;
        float deltaTime;

        bool blockedUp = false;
        bool blockedDown = false;
        bool blockedLeft = false;
        bool blockedRight = false;
        bool blockedForward = false;
        bool blockedBackward = false;

        vec3 directions[6] = {
            vec3(1, 0, 0),
            vec3(0, -1, 0),
            vec3(0, 0, -1),
            vec3(-1, 0, 0),
            vec3(0, 1, 0),
            vec3(0, 0, 1)
        };

        bool hitObject = false;

        void checkForCollisions(vector<Model>, btCollisionWorld* collisionWorld);

};


#endif