#ifndef PLAYER_HPP
#define PLAYER_HPP

#include "main.hpp"

class Camera;

class Player : public Camera {
    public:
        Player(float, float, float, float, float, vec3, vec4, float);

        void updatePlayer(float, const Uint8*, float, float, btCollisionWorld*);

        vec3 position;

        void movePlayer(Axis, float);

    private:
        vec4 size;
        vec3 velocity, prevPos, playerTranslationVec;
        float deltaTime, speed, reach;

        void interact(btCollisionWorld*);
};


#endif