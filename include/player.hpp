#ifndef PLAYER_HPP
#define PLAYER_HPP

#include "main.hpp"

class Camera;
class Entity;

class Player : public Entity, public Camera {
    public:
        Player(float, float, float, float, float, vec3, vec4, float, float);

        enum ITEM {FOV, ASPECT, NEAR, FAR, POSITION, CAMERAPOSITION, UP, FRONT, RIGHT};
        variant<float, vec3> getItem(ITEM);

        void updatePlayer(float, const Uint8*, float, float);

        void movePlayer(float);

    private:
        float deltaTime, movementForce, reach;

        void interact(btCollisionWorld*);
};


#endif