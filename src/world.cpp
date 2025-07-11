#include "main.hpp"

Chunk::Chunk(worldGenInfo genInfo, vec3 origin, int worldHeight) : genInfo(genInfo), origin(origin), grid(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE, VOXEL::EMPTY), worldHeight(worldHeight) {
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int z = 0; z < CHUNK_SIZE; z++){
            int height = worldHeight * genInfo.heightMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * 0.01, ((origin.z * CHUNK_SIZE) + z) * 0.01, 4);
            float temp = genInfo.tempMap.octave2D_11(((origin.x * CHUNK_SIZE) + x) * 0.005, ((origin.z * CHUNK_SIZE) + z)  * 0.005, 4);
            float moistness = genInfo.moistureMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * 0.005, ((origin.z * CHUNK_SIZE) + z)  * 0.005, 4);

            VOXEL _blockType = VOXEL::EMPTY;
            
            if((temp >= 0.3f && temp <= 1.0f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = Chunk::VOXEL::HOTDRY;}        //hd
            else if ((temp >= 0.3f && temp <= 1.0f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = Chunk::VOXEL::HOTNORM;}    //hn
            else if ((temp >= 0.3f && temp <= 1.0f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = Chunk::VOXEL::HOTWET;}  //hw
            else if ((temp > -0.3f && temp < 0.3f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = Chunk::VOXEL::NORMDRY;}    //nd
            else if ((temp > -0.3f && temp < 0.3f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = Chunk::VOXEL::NORM;}      //nn
            else if ((temp > -0.3f && temp < 0.3f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = Chunk::VOXEL::NORMWET;}    //nw
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = Chunk::VOXEL::COLDDRY;}  //cd
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = Chunk::VOXEL::COLDNORM;}    //cn
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = Chunk::VOXEL::COLDWET;}  //cw

            for(int y = 0; y < CHUNK_SIZE; y++){
                if(((origin.y * CHUNK_SIZE) + y) > height) continue;
                
                grid[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE] = _blockType;
            }
        }
    }
}

const unordered_map<vec3, vector<vec3>, vecHash> Chunk::faces {
            {vec3(0, 0, 1), {vec3(-1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1), vec3(1, -1, 1)}}, 
            {vec3(0, 1, 0), {vec3(-1, 1, -1), vec3(1, 1, 1), vec3(1, 1, -1), vec3(-1, 1, 1)}},
            {vec3(1, 0, 0), {vec3(1, -1, -1), vec3(1, 1, 1), vec3(1, -1, 1), vec3(1, 1, -1)}},
            {vec3(0, 0, -1), {vec3(-1, -1, -1), vec3(1, 1, -1), vec3(1, -1, -1), vec3(-1, 1, -1)}},
            {vec3(0, -1, 0), {vec3(-1, -1, -1),vec3(1, -1, 1), vec3(-1, -1, 1), vec3(1, -1, -1)}},
            {vec3(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}
        };

const map<Chunk::VOXEL, vec3> Chunk::blockType {
            {Chunk::VOXEL::HOTDRY, vec3(1, 0, 0)},
            {Chunk::VOXEL::HOTNORM, vec3(1, 0.3, 0)},
            {Chunk::VOXEL::HOTWET, vec3(0.8, 0.29, 0.16)},
            {Chunk::VOXEL::NORMDRY, vec3(1, 0.65, 0)},
            {Chunk::VOXEL::NORM, vec3(0.93, 0.76, 0)},
            {Chunk::VOXEL::NORMWET, vec3(0.76, 0.9, 0.027)},
            {Chunk::VOXEL::COLDDRY, vec3(0.18, 0.87, 0.27)},
            {Chunk::VOXEL::COLDNORM, vec3(0.15, 0.95, 0.45)},
            {Chunk::VOXEL::COLDWET, vec3(0.062, 0.97, 0.85)}
        };

const vector<vec3> Chunk::neighbors = {vec3(0, 0, 1),
                                       vec3(0, 1, 0),
                                       vec3(1, 0, 0),
                                       vec3(0, 0, -1),
                                       vec3(0, -1, 0),
                                       vec3(-1, 0, 0)};

void Chunk::genMesh(const vector<Chunk>& chunks){
    vector<vec3> verts, colors, normals;
    vector<unsigned int> inds;
    VOXEL neighborValue = VOXEL::EMPTY;
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int y = 0; y < CHUNK_SIZE; y++){
            for(int z = 0; z < CHUNK_SIZE; z++){
                VOXEL voxel = grid.at(x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE);
                if(voxel == VOXEL::EMPTY) continue;
                vec3 position(x, y, z);
                for(const auto& neighbor : neighbors){
                    vec3 _neighbor = position + neighbor;
                    if(checkOutOfBounds(_neighbor)){
                        vec3 _origin = vec3(floor(((origin.x * CHUNK_SIZE) + _neighbor.x)/CHUNK_SIZE),
                                            floor(((origin.y * CHUNK_SIZE) + _neighbor.y)/CHUNK_SIZE),
                                            floor(((origin.z * CHUNK_SIZE) + _neighbor.z)/CHUNK_SIZE));
                        for(const auto& chunk : chunks){
                            if(chunk.origin != _origin) continue;

                            vec3 _voxel = vec3((int)((origin.x * CHUNK_SIZE) + _neighbor.x) % CHUNK_SIZE,
                                               (int)((origin.y * CHUNK_SIZE) + _neighbor.y) % CHUNK_SIZE,   
                                               (int)((origin.z * CHUNK_SIZE) + _neighbor.z) % CHUNK_SIZE);

                                neighborValue = chunk.grid.at(_voxel.x + _voxel.y * CHUNK_SIZE + _voxel.z * CHUNK_SIZE * CHUNK_SIZE);
                        }
                    } else {neighborValue = grid.at(_neighbor.x + _neighbor.y * CHUNK_SIZE + _neighbor.z * CHUNK_SIZE * CHUNK_SIZE);}
                        
                    if(neighborValue != VOXEL::EMPTY) continue;
                    GLuint idxStart = verts.size();

                    vec3 _position = ((origin * static_cast<float>(CHUNK_SIZE)) + (position)) * static_cast<float>(VOXEL_SIZE);
                    vector<vec3> face = faces.at(neighbor);
                    vec3 color = blockType.at(voxel);

                    normals.insert(normals.end(), 4, neighbor);
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

    world.reserve(renderDist * renderDist);

    for(int x = 0; x < renderDist; x++){
            for(int z = 0; z < renderDist; z++){
                cout << roundf(((float)((x * renderDist) + z) / (float)(renderDist * renderDist)) * 100.0f) << "% generating: (" << x << ", " << z << ")\n";
                world.emplace_back(Chunk(worldSeed, vec3(x, 0, z), worldHeight));
                world.emplace_back(Chunk(worldSeed, vec3(x, 1, z), worldHeight));
                world.emplace_back(Chunk(worldSeed, vec3(x, 2, z), worldHeight));
            }
    }

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos) {
    glEnable(GL_DEPTH_TEST);

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
    for(auto& chunk : world){
        if(!chunk.dirty) continue;
        chunk.genMesh(world);
    }
    prepAndCombineBuffers(playerPos);
}

void World::prepAndCombineBuffers(vec3 playerPos) {
    if(buffered) return;
    buffered = true;
    vector<vec3> verts, normals, colors;
    vector<unsigned int> inds;
    size_t lastVertsSize = 0;

    Box renderBox(playerPos - vec3(renderDist), playerPos + vec3(renderDist));

    for(auto& chunk : world){
        if(!renderBox.isInside(chunk.origin)) continue;

        Mesh mesh = chunk.mesh;

        verts.insert(verts.end(), mesh.verts.begin(), mesh.verts.end());
        normals.insert(normals.end(), mesh.normals.begin(), mesh.normals.end());
        colors.insert(colors.end(), mesh.colors.begin(), mesh.colors.end());
        for(const auto& ind : mesh.inds){
            inds.push_back(ind + lastVertsSize);
        }

        chunk.buffered = true;

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