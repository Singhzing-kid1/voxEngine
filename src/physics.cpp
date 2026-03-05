#include "main.hpp"

Physics::Physics(float gravity) : gravity(gravity) {
    btBroadphaseInterface* broadphase = new btDbvtBroadphase();
    btDefaultCollisionConfiguration* config = new btDefaultCollisionConfiguration();
    btCollisionDispatcher* dispatcher = new btCollisionDispatcher(config);
    collisionWorld = new btCollisionWorld(dispatcher, broadphase, config);


}

void Physics::step(bool applyGravity){
    for(auto& entity : entities){
        vec3 weight = entity.get().mass * gravity * gravityDirection;
        if (applyGravity){
            entity.get().addForce(weight);
        }

        for(auto direction  : directions){
            bool hit = collisionCheck(entity.get().size, entity.get().position, direction, {entity.get().rYaw, entity.get().rPitch});

            if(!hit) continue;

            float isolatedVelocityDirection = dot(direction, entity.get().velocity);
            float isolatedNetForceDirection = dot(direction, entity.get().netForce);

            float velocityAngle = degrees(angle(direction, entity.get().velocity));
            float forceAngle = degrees(angle(direction, entity.get().netForce));

            if(velocityAngle < 90){
                entity.get().velocity -= isolatedVelocityDirection * direction;
            }


            entity.get().addForce(-isolatedNetForceDirection * direction);
        }
    }
}

bool Physics::collisionCheck(vec3 size, vec3 position, vec3 direction, pair<float, float> orientation){
    quat qYaw = angleAxis(orientation.first, vec3(0.0f, 1.0f, 0.0f));

    vec3 rotatedDirection = normalize(rotate(qYaw, direction));

    btVector3 rayStart(position.x, position.y, position.z);
    btVector3 rayEnd(position.x + rotatedDirection.x, position.y + rotatedDirection.y, position.z + rotatedDirection.z);

    btCollisionWorld::ClosestRayResultCallback rayCallback(rayStart, rayEnd);
    collisionWorld->rayTest(rayStart, rayEnd, rayCallback);
    
    return rayCallback.hasHit();
}



