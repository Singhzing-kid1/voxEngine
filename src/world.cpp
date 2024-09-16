#include "main.hpp"

World::World(int chunkSize, int worldHeight){
    this->chunkSize = chunkSize;
    this->worldHeight = worldHeight;
    initialize();
    this->init();
}

World::~World() {
    // Clean up OpenGL resources
    for (auto& chunk : chunks) {
        glDeleteVertexArrays(1, &chunk.vao);
        glDeleteBuffers(1, &chunk.vbo);
        glDeleteBuffers(1, &chunk.ebo);
    }
    uninitialize();
}

void World::init() {
    // Create chunks
    Chunk chunk;

    chunk.voxelGrid = FloatGrid::create(-1.0f);
    chunk.voxelGrid->setTransform(math::Transform::createLinearTransform(1.0));

    FloatGrid::Accessor accessor = chunk.voxelGrid->getAccessor();

    queue<worldCell> queue;

    int height = 10;
    Coord start(0, height, 0);
    accessor.setValueOn(start, 1.0f);
    queue.push(worldCell(start, height));

    while(!queue.empty()){
        worldCell current = queue.front();
        queue.pop();

        int currentHeight = current.height;

        for(int y = 0; y < currentHeight; y++){
            Coord setToHeight(current.coord.x(), y, current.coord.z());
            accessor.setValueOn(setToHeight, 1.0f);
        }

        if(currentHeight <= 0) continue;

        vector<worldCell> neighbors = {
            worldCell(Coord(current.coord.x() + 1, 0, current.coord.z()), currentHeight - 1),  // Right
            worldCell(Coord(current.coord.x(), 0, current.coord.z() + 1), currentHeight - 1),  // Down
            worldCell(Coord(current.coord.x() - 1, 0, current.coord.z()), currentHeight - 1),  // Left
            worldCell(Coord(current.coord.x(), 0, current.coord.z() - 1), currentHeight - 1)   // Up
        };

        for(worldCell& neighbor : neighbors){
            if (neighbor.coord.x() >= 0 && neighbor.coord.x() < chunkSize && neighbor.coord.z() >= 0 && neighbor.coord.z() < chunkSize) {
                Coord neighborCoord(neighbor.coord.x(), neighbor.height, neighbor.coord.z());
                if (accessor.getValue(neighborCoord) == -1) {
                    accessor.setValueOn(neighborCoord, 1.0f);
                    
                    queue.push(neighbor);
                }
            }
        }
    } 


    chunk.needsUpdate = true;
    chunk.chunkPos = Coord(0, 0, 0);

    chunks.push_back(chunk);
}

void World::update() {
    for (auto& chunk : chunks) {
        if (chunk.needsUpdate) {
            generateChunkMesh(chunk);
            Chunk data;

            data = generateChunkMesh(chunk);
            data = setupMeshBuffers(data);

            chunk = data;
            chunk.needsUpdate = false;
        }
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos) {
    for (auto& chunk : chunks) {
        shader.use();
        shader.setUniform("view", view);
        shader.setUniform("projection", projection);
        shader.setUniform("model", model);
        shader.setUniform("color", vec3(1.0f, 1.0f, 1.0f));
        shader.setUniform("lightPosition", vec3(8, 5, 5));
        shader.setUniform("viewPos", cameraPos);

        glBindVertexArray(chunk.vao);
        glDrawElements(GL_TRIANGLES, chunk.indicies.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

World::Chunk World::generateChunkMesh(Chunk chunk) {
    Chunk data = chunk;

    FloatGrid::Accessor accessor = data.voxelGrid->getAccessor();

    auto addQuad = [&](const Vec3s& v0, const Vec3s& v1, const Vec3s& v2, const Vec3s& v3, Vec3s normal){
        GLuint idxStart = data.verticies.size();

        data.normals.push_back(normal);
        data.normals.push_back(normal);
        data.normals.push_back(normal);
        data.normals.push_back(normal);

        data.verticies.push_back(v0);
        data.verticies.push_back(v1);
        data.verticies.push_back(v2);
        data.verticies.push_back(v3);

        data.indicies.push_back(idxStart + 0);
        data.indicies.push_back(idxStart + 1);
        data.indicies.push_back(idxStart + 2);
    
        data.indicies.push_back(idxStart + 0);
        data.indicies.push_back(idxStart + 3);
        data.indicies.push_back(idxStart + 1);
    };


    for(auto iter = data.voxelGrid->beginValueOn(); iter; ++iter){
        const Coord& coord = iter.getCoord();
        if (accessor.getValue(coord) != 1.0f) continue;

        Vec3s voxelCenter = voxelToWorld(coord, data.chunkPos, 1.0f, 16);

        if(isSurfaceVoxel(coord, data.voxelGrid)){
            if (accessor.getValue(coord.offsetBy(1, 0, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + Vec3s(0.5f, -0.5f, -0.5f),
                    voxelCenter + Vec3s(0.5f,  0.5f,  0.5f),
                    voxelCenter + Vec3s(0.5f, -0.5f,  0.5f),
                    voxelCenter + Vec3s(0.5f,  0.5f, -0.5f),
                    Vec3s(1.0f, 0.0f, 0.0f)
                );
            }
            
            if (accessor.getValue(coord.offsetBy(-1, 0, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + Vec3s(-0.5f, -0.5f, -0.5f),
                    voxelCenter + Vec3s(-0.5f,  0.5f,  0.5f),
                    voxelCenter + Vec3s(-0.5f,  0.5f, -0.5f),
                    voxelCenter + Vec3s(-0.5f, -0.5f,  0.5f),
                    Vec3s(-1.0f, 0.0f, 0.0f)
                );
            }
            
            if (accessor.getValue(coord.offsetBy(0, 1, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + openvdb::Vec3s(-0.5f, 0.5f, -0.5f),
                    voxelCenter + openvdb::Vec3s( 0.5f, 0.5f,  0.5f),
                    voxelCenter + openvdb::Vec3s( 0.5f, 0.5f, -0.5f),
                    voxelCenter + openvdb::Vec3s(-0.5f, 0.5f,  0.5f),
                    Vec3s(0.0f, 1.0f, 0.0f)
                );
            }

            if (accessor.getValue(coord.offsetBy(0, -1, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + Vec3s(-0.5f, -0.5f, -0.5f),
                    voxelCenter + Vec3s( 0.5f, -0.5f,  0.5f),
                    voxelCenter + Vec3s(-0.5f, -0.5f,  0.5f),
                    voxelCenter + Vec3s( 0.5f, -0.5f, -0.5f),
                    Vec3s(0.0f, -1.0f, 0.0f)
                );
            }
            
            if (accessor.getValue(coord.offsetBy(0, 0, 1)) == -1.0f) {
                addQuad(
                    voxelCenter + Vec3s(-0.5f, -0.5f, 0.5f),
                    voxelCenter + Vec3s( 0.5f,  0.5f, 0.5f),
                    voxelCenter + Vec3s(-0.5f,  0.5f, 0.5f),
                    voxelCenter + Vec3s( 0.5f, -0.5f, 0.5f),
                    Vec3s(0.0f, 0.0f, -1.0f)
                );
            }

            if (accessor.getValue(coord.offsetBy(0, 0, -1)) == -1.0f) {
                addQuad(
                    voxelCenter + Vec3s(-0.5f, -0.5f, -0.5f),
                    voxelCenter + Vec3s( 0.5f,  0.5f, -0.5f),
                    voxelCenter + Vec3s( 0.5f, -0.5f, -0.5f),
                    voxelCenter + Vec3s(-0.5f,  0.5f, -0.5f),
                    Vec3s(0.0f, 0.0f, -1.0f)
                );
            }
        }
    }


    return data;
}

World::Chunk World::setupMeshBuffers(Chunk chunk) {
    Chunk data = chunk;

    glGenVertexArrays(1, &data.vao);
    glGenBuffers(1, &data.vbo);
    glGenBuffers(1, &data.ebo);

    glBindVertexArray(data.vao);

    glBindBuffer(GL_ARRAY_BUFFER, data.vbo);
    glBufferData(GL_ARRAY_BUFFER, data.verticies.size() * sizeof(Vec3s) * 2, nullptr, GL_STATIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, data.verticies.size() * sizeof(Vec3s), data.verticies.data());

    glBufferSubData(GL_ARRAY_BUFFER, data.verticies.size() * sizeof(Vec3s), data.normals.size() * sizeof(Vec3s), data.normals.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (data.verticies.size() * sizeof(Vec3s)));
    glEnableVertexAttribArray(1);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, data.ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, data.indicies.size() * sizeof(unsigned int), data.indicies.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);

    data.newChunk = false;

    return data;
}

bool World::isSurfaceVoxel(Coord coord, FloatGrid::Ptr voxelGrid){
    FloatGrid::Accessor accessor = voxelGrid->getAccessor();

    float currentVoxelValue = accessor.getValue(coord);

    vector<Coord> directions = {Coord(0, 0, 1),
                                Coord(0, 1, 0),
                                Coord(1, 0, 0),
                                Coord(0, 0, -1),
                                Coord(0, -1, 0),
                                Coord(-1, 0, 0)};

    for(auto direction : directions){
        Coord offset;

        if((coord.x() >= 0 || coord.y() >= 0 || coord.z() >= 0)){
            offset = coord.offsetBy(direction.x(), direction.y(), direction.z());

            if(accessor.getValue(offset) == -1){
                return true;
            }
    

        }
    }

    return false;
}


Vec3s World::voxelToWorld(Coord coord, Coord chunkPos, float voxelSize, float chunkSize){
    Vec3i voxelCoord = coord.asVec3i();
    Vec3s chunkPosCoord = chunkPos.asVec3s(); 
    Vec3s worldPos;

    worldPos.x() = chunkPos.x() * chunkSize + voxelCoord.x() * voxelSize;
    worldPos.y() = chunkPos.y() * chunkSize + voxelCoord.y() * voxelSize;
    worldPos.z() = chunkPos.z() * chunkSize + voxelCoord.z() * voxelSize;

    return worldPos;
}
