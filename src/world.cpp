#include "main.hpp"

Chunk::Chunk(worldGenInfo genInfo, vec3 origin, int worldHeight) : origin(origin), grid(CHUNK_SIZE, vector<vector<Voxel>>(CHUNK_SIZE, vector<Voxel>(CHUNK_SIZE))) {
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int z = 0; z < CHUNK_SIZE; z++){
            int height = worldHeight * genInfo.heightMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * 0.01, ((origin.z * CHUNK_SIZE) + z) * 0.01, 4);
            for(int y = 0; y < CHUNK_SIZE; y++){
                if(((origin.y * CHUNK_SIZE) + y) > height) continue;
                grid[x][y][z] = Voxel(1);
            }
        }
    }

    genMesh();
}

void Chunk::genMesh(){
    vector<vec3> verts, colors, normals;
    vector<unsigned int> inds;
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int y = 0; y < CHUNK_SIZE; y++){
            for(int z = 0; z < CHUNK_SIZE; z++){
                Voxel voxel = grid.at(x).at(y).at(z);
                if(voxel.value == -1) continue;
                vec3 position(x, y, z);
                for(const auto& neighbor : voxel.neighbors){
                    vec3 _neighbor = position + neighbor;
                    if(checkOutOfBounds(_neighbor)) continue;
                    int neighborValue = grid.at(_neighbor.x).at(_neighbor.y).at(_neighbor.z).value;
                    if(neighborValue != -1) continue;
                    GLuint idxStart = verts.size();

                    vec3 _position = (origin * static_cast<float>(CHUNK_SIZE)) + (position * static_cast<float>(VOXEL_SIZE));
                    vector<vec3> face = faces.at(neighbor);
                    vec3 color = vec3(0.7f);

                    normals.insert(normals.end(), 4, (vec3)neighbor);
                    verts.insert(verts.end(), {_position + (face[0] * halfVoxelSize), _position + (face[1] * halfVoxelSize), _position + (face[2] * halfVoxelSize), _position + (face[3] * halfVoxelSize)});
                    inds.insert(inds.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 0, idxStart + 3, idxStart + 1});
                    colors.insert(colors.end(), 4, color);
                }
            }
        }
    }
    dirty = false;

    mesh.verts.assign(verts.begin(), verts.end());
    mesh.inds.assign(inds.begin(), inds.end());
    mesh.colors.assign(colors.begin(), colors.end());
    mesh.normals.assign(normals.begin(), normals.end());
}

Mesh Chunk::getMesh(int lod){
    return mesh;
}

bool Chunk::checkOutOfBounds(vec3 coord){
    return coord.x < 0 || coord.y < 0 || coord.z < 0 || coord.x >= CHUNK_SIZE || coord.y >= CHUNK_SIZE || coord.z >= CHUNK_SIZE;
}

World::World(float voxelSize, int worldDimension, int worldHeight, int renderDist, String seed) : worldHeight(worldHeight), worldDimension(worldDimension), renderDist(renderDist) {
    unsigned int seed1, seed2, seed3;
    int len = seed.length() / 3;

    seed1 = hash(seed.substr(0, len));
    seed2 = hash(seed.substr(len, len));
    seed3 = hash(seed.substr(len*2, len));

    worldSeed.heightMap.reseed(seed1);
    worldSeed.moistureMap.reseed(seed2);
    worldSeed.tempMap.reseed(seed3);

    for(int x = 0; x < renderDist; x++){
        for(int y = 0; y < renderDist; y++){
            for(int z = 0; z < renderDist; z++){
                world.push_back(Chunk(worldSeed, vec3(x, y, z), worldHeight));
            }
        }
    }

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos) {
    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("lightPosition", vec3(0, 30, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(vao);
    glDrawElements(GL_TRIANGLES, indsSize, GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(vec3 playerPos) {
    prepAndCombineBuffers(playerPos);
}

void World::prepAndCombineBuffers(vec3 playerPos) {
    if(buffered) return;
    buffered = true;
    vector<vec3> verts, normals, colors;
    vector<unsigned int> inds;
    size_t lastVertsSize = 0;
    for(size_t x = 0; x < world.size(); x++){
        Chunk chunk = world.at(x);
        
        Mesh mesh = chunk.mesh;

        verts.insert(verts.end(), mesh.verts.begin(), mesh.verts.end());
        normals.insert(normals.end(), mesh.normals.begin(), mesh.normals.end());
        colors.insert(colors.end(), mesh.colors.begin(), mesh.colors.end());
        for(const auto& ind : mesh.inds){
            inds.push_back(ind + lastVertsSize);
        }

        lastVertsSize += mesh.verts.size();
    }


    indsSize = inds.size();

    glBindVertexArray(vao);

    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, (verts.size() + normals.size() + colors.size()) * sizeof(vec3), nullptr, GL_STATIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, verts.size() * sizeof(vec3), verts.data());
    glBufferSubData(GL_ARRAY_BUFFER, verts.size() * sizeof(vec3), normals.size() * sizeof(vec3), normals.data());
    glBufferSubData(GL_ARRAY_BUFFER, (verts.size() + normals.size()) * sizeof(vec3), colors.size() * sizeof(vec3), colors.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (verts.size() * sizeof(vec3)));
    glEnableVertexAttribArray(1);

    glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) ((verts.size() + normals.size()) * sizeof(vec3)));
    glEnableVertexAttribArray(2);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, indsSize * sizeof(unsigned int), inds.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);
}

int World::hash(String str){
    int hash = 0;
    for(auto c : str){
        hash += (int)c;
    }

    return hash;
}