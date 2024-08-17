#include "main.hpp"

Model::Model(const char* modelPath, float size, vec3 chunkSpaceTransform, vec3 chunk, bool shouldScalePositionBasedOnScaleOfVoxel, string id, bool canCollide, btCollisionWorld* collisionWorld){
    ifstream model(modelPath);
    string line;
    vector<string> data;

    this->chunkSpaceTransform = chunkSpaceTransform;
    this->chunk = chunk;
    this->size = size;
    this->shouldScalePosition = shouldScalePositionBasedOnScaleOfVoxel;
    this->ID = id;
    this->canCollide = canCollide;

    while(getline(model, line)){
        if(line != "##MODEL" && line != "##EOF"){
            data.push_back(line);
        } else if(line == "##EOF"){
            break;
        }
    }

    vector<string> color, location;

    for(const auto &d : data){
        string temp = d;
        replace(temp.begin(), temp.end(), '|', ' ');

        stringstream ss(temp);
        int selector = 0;
        temp = "";
        while(ss >> temp){
            if(selector == 0){
                location.push_back(temp);
                selector = 1;
            } else if(selector == 1){
                color.push_back(temp);
                selector = 0;
            }
        }
    }

    for(const auto &l: location){
        string temp = l;
        replace(temp.begin(), temp.end(), ',', ' ');

        stringstream ss(temp);
        float tempF;
        vec3 tempV;
        for(int x = 0; x < 3; x++){
            ss >> tempF;
            tempF = tempF;
            tempV[x] = tempF;
        }
        this->position.push_back(tempV);
    }

    for(const auto &c: color){
        string temp = c;
        replace(temp.begin(), temp.end(), ',', ' ');

        stringstream ss(temp);
        float tempF;
        vec3 tempV;
        for(int x = 0; x < 3; x++){
            ss >> tempF;
            tempF = tempF/255;
            tempV[x] = tempF;
        }
        this->color.push_back(tempV);
    }


    for(const auto &p : this->position){
        btVector3 halfSize(size, size, size);
        btCollisionShape* boxShape = new btBoxShape(halfSize);

        btTransform transform;
        transform.setIdentity();
        transform.setOrigin(btVector3(p.x, p.y, p.z) * 2 * size + btVector3(chunk.x, chunk.y, chunk.z) * 34 + btVector3(chunkSpaceTransform.x, chunkSpaceTransform.y, chunkSpaceTransform.z));

        btCollisionObject* collisionObject = new btCollisionObject();
        collisionObject->setCollisionShape(boxShape);
        collisionObject->setWorldTransform(transform);

        canCollide ? collisionWorld->addCollisionObject(collisionObject) : (void)0;
    }

    float vertices[] = {
        1.0f, 1.0f, 1.0f, 
        1.0f, -1.0f, 1.0f, 
        -1.0f, 1.0f, 1.0f, 
        -1.0f, -1.0f, 1.0f,
        1.0f, 1.0f, -1.0f, 
        1.0f, -1.0f, -1.0f, 
        -1.0f, 1.0f, -1.0f, 
        -1.0f, -1.0f, -1.0f
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

    for(size_t x = 0; x < sizeof(vertices)/sizeof(float); x++){
        vertices[x] *= size;
    }
    

    glGenVertexArrays(1, &this->vao);
    glGenBuffers(1, &this->vbo);
    glGenBuffers(1, &this->ebo);

    glBindVertexArray(this->vao);

    glBindBuffer(GL_ARRAY_BUFFER, this->vbo);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, this->ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);

    glBindVertexArray(0);



}

unsigned int Model::getVao(){
    return this->vao;
}

vector<vec3> Model::getPositions(){
    return this->position;
}

vector<vec3> Model::getColors(){
    return this->color;
}

vec3 Model::getChunkCoord(){
    return this->chunk;
}

void Model::setChunkCoord(vec3 chunk){
    this->chunk = chunk;
}

string Model::getId(){
    return this->ID;
}

bool Model::getShouldScalePositionBool(){
    return this->shouldScalePosition;
}

bool Model::getCanCollide(){
    return this->canCollide;
}

vector<vec3> Model::getVoxelPointsInGlobal(){
    vector<vec3> result;
    for(auto p : this->position){
        vec3 out;
        out.x = p.x * 2 * this->size + this->chunk.x * 34 + this->chunkSpaceTransform.x;
        out.y = p.y * 2 * this->size + this->chunk.y * 34 + this->chunkSpaceTransform.y;
        out.z = p.z * 2 * this->size + this->chunk.z * 34 + this->chunkSpaceTransform.z;

        result.push_back(out);
    }

    return result;
}

float Model::getSize(){
    return this->size;
}

void Model::render(Shader shader, mat4 model, mat4 view, mat4 projection){
        for(size_t x = 0; x < this->getPositions().size(); x++){
            shader.use();
            shader.setUniform("model", model);
            shader.setUniform("view", view);
            shader.setUniform("projection", projection);

            shader.setUniform("position", this->position[x]);
            shader.setUniform("chunkSpaceTransform", this->chunkSpaceTransform);
            shader.setUniform("chunk", this->chunk);

            this->shouldScalePosition ? shader.setUniform("size", this->size) : (void)0;

            shader.setUniform("color", this->color[x]);

            glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
            glEnableVertexAttribArray(0);

            glBindVertexArray(this->vao);
            glDrawElements(GL_TRIANGLES, 36, GL_UNSIGNED_INT, 0);
            glBindVertexArray(0);
        }
}