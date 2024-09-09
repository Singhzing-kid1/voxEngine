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

    int height = 8;     // Let's create some height variation

    
    for(int x = 0; x < chunkSize; x++){
        for(int z = 0; z < chunkSize; z++){
            for(int y = 0; y < height; y++){
                Coord xyz(x, y, z);
                if (y < (height / 2)) {
                    accessor.setValueOn(xyz, 1.0f);  // Inside
                } else {
                    accessor.setValueOn(xyz, -1.0f); // Outside
                }
            }
        }
    }

    chunk.needsUpdate = true;
    chunk.newChunk = true;

    chunks.push_back(chunk);
}

void World::update() {
    for (auto& chunk : chunks) {
        if (chunk.needsUpdate) {
            generateChunkMesh(chunk);
            Chunk data;
            if(chunk.newChunk){
                data = generateChunkMesh(chunk);
                data = setupMeshBuffers(data);
            }
            chunk = data;
            chunk.needsUpdate = false;
        }
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader) {
    for (auto& chunk : chunks) {
        shader.use();
        shader.setUniform("view", view);
        shader.setUniform("projection", projection);
        shader.setUniform("model", model);
        shader.setUniform("color", vec3(1.0f, 1.0f, 1.0f));

        glBindVertexArray(chunk.vao);
        glDrawElements(GL_TRIANGLES, chunk.indicies.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

World::Chunk World::generateChunkMesh(Chunk chunk) {
    vector<Vec3s> points;
    vector<Vec4I> quads;

    Chunk data = chunk;

    tools::volumeToMesh(*data.voxelGrid, points, quads);

    for (const auto& quad : quads) {
        data.indicies.push_back(quad[0]);
        data.indicies.push_back(quad[1]);
        data.indicies.push_back(quad[2]);
        data.indicies.push_back(quad[0]);
        data.indicies.push_back(quad[2]);
        data.indicies.push_back(quad[3]);
    }

    data.verticies = points;

    vector<Vec3s> normals(points.size(), Vec3s(0.0f, 0.0f, 0.0f));

/*     for(size_t i = 0; i < data.indicies.size(); i+=3){
        Vec3s v1 = points[data.indicies[i+1]] - points[data.indicies[i]];
        Vec3s v2 = points[data.indicies[1+2]] - points[data.indicies[i]];
        Vec3s normal = v1.cross(v2).unit();
        normals[i] += normal;
        normals[i + 1] += normal;
        normals[i + 2] += normal;
    } */

    data.normals = normals;
    
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

 /*    glBufferSubData(GL_ARRAY_BUFFER, data.verticies.size() * sizeof(Vec3s), data.normals.size() * sizeof(Vec3s), data.normals.data()); */

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

 /*    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (data.verticies.size() * sizeof(Vec3s)));
    glEnableVertexAttribArray(1); */

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, data.ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, data.indicies.size() * sizeof(unsigned int), data.indicies.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);

    data.newChunk = false;

    return data;
}
