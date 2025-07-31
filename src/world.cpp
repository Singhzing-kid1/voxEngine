#include "main.hpp"

Chunk::Chunk(noiseMaps maps, vec2 origin, int worldHeight) : maps(maps), origin(origin), worldHeight(worldHeight), grid(CHUNK_SIZE, vector<vector<VOXEL>>(worldHeight, vector<VOXEL>(CHUNK_SIZE, VOXEL::EMPTY))){
    for(int x = 0; x < (int)CHUNK_SIZE; x++){
        for(int z = 0; z < (int)CHUNK_SIZE; z++){
            int height = (int)MAX_WORLD_GENERATION_HEIGHT * maps.heightMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * heightMapScalar, ((origin.y * CHUNK_SIZE) + z) * heightMapScalar, perlinNoiseOctaveAmount);
            float moistness = maps.moistureMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.y * CHUNK_SIZE) + z) * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);
            float temp = maps.tempMap.octave2D_11(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.y * CHUNK_SIZE) + z) * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);

            VOXEL _blockType = VOXEL::EMPTY;

            if((temp >= tempHotMin && temp <= tempHotMax) && (moistness >= dryMin && moistness <= dryMax)) {_blockType = Chunk::VOXEL::HOTDRY;}        //hd
            else if ((temp >= tempHotMin && temp <= tempHotMax) && (moistness > normMin && moistness < normMax)) {_blockType = Chunk::VOXEL::HOTNORM;}    //hn
            else if ((temp >= tempHotMin && temp <= tempHotMax) && (moistness >= wetMin && moistness <= wetMax)) {_blockType = Chunk::VOXEL::HOTWET;}  //hw
            else if ((temp > tempNormMin && temp < tempNormMax) && (moistness >= dryMin && moistness <= dryMax)) {_blockType = Chunk::VOXEL::NORMDRY;}    //nd
            else if ((temp > tempNormMin && temp < tempNormMax) && (moistness > normMin && moistness < normMax)) {_blockType = Chunk::VOXEL::NORM;}      //nn
            else if ((temp > tempNormMin && temp < tempNormMax) && (moistness >= wetMin && moistness <= wetMax)) {_blockType = Chunk::VOXEL::NORMWET;}    //nw
            else if ((temp >= tempColdMin && temp <= tempColdMax) && (moistness >= dryMin && moistness <= dryMax)) {_blockType = Chunk::VOXEL::COLDDRY;}  //cd
            else if ((temp >= tempColdMin && temp <= tempColdMax) && (moistness > normMin && moistness < normMax)) {_blockType = Chunk::VOXEL::COLDNORM;}    //cn
            else if ((temp >= tempColdMin && temp <= tempColdMax) && (moistness >= wetMin && moistness <= wetMax)) {_blockType = Chunk::VOXEL::COLDWET;}  //cw
            
            for(int y = 0; y < worldHeight; y++){
                if(y > height) continue;
                grid[x][y][z] = _blockType;
            }
        }
    }
}

const unordered_map<vec3, vector<vec3>, vec3Hash> Chunk::faces {
            {vec3(0, 0, 1), {vec3(-1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1), vec3(1, -1, 1)}}, 
            {vec3(0, 1, 0), {vec3(-1, 1, -1), vec3(1, 1, 1), vec3(1, 1, -1), vec3(-1, 1, 1)}},
            {vec3(1, 0, 0), {vec3(1, -1, -1), vec3(1, 1, 1), vec3(1, -1, 1), vec3(1, 1, -1)}},
            {vec3(0, 0, -1), {vec3(-1, -1, -1), vec3(1, 1, -1), vec3(1, -1, -1), vec3(-1, 1, -1)}},
            {vec3(0, -1, 0), {vec3(-1, -1, -1),vec3(1, -1, 1), vec3(-1, -1, 1), vec3(1, -1, -1)}},
            {vec3(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}
        };

const map<Chunk::VOXEL, vec3> Chunk::blockType { // if not clear these are color values, i might refactor to make it more self documenting
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

void Chunk::genMesh(unordered_map<vec2, Chunk, vec2Hash>& world){
    vector<vec3> vertices; vector<vec3> colors; vector<vec3> normals;
    vector<unsigned int> indices;

    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int y = 0; y < worldHeight; y++){
            for(int z = 0; z < CHUNK_SIZE; z++){
                VOXEL& c_voxel = grid.at(x).at(y).at(z);
                vec3 c_coord(x, y, z);

                if(c_voxel == VOXEL::EMPTY || c_voxel == VOXEL::AIR) continue;
                for(const auto& neighbor : neighbors){
                    vec3 n_coord = c_coord + neighbor;
                    VOXEL n_voxel;

                    if(outOfBounds(n_coord)){
                        vec2 n_chunkCoord = ((origin * (float)CHUNK_SIZE) + n_coord.xz()) / (float)CHUNK_SIZE;

                        try{    
                            Chunk& n_chunk = world.at(n_chunkCoord);
                            n_coord = mod(((vec3(origin.x, 0, origin.y) * (float)CHUNK_SIZE) + n_coord), (float)CHUNK_SIZE);
                            n_voxel = n_chunk.grid.at(n_coord.x).at(n_coord.y).at(n_coord.z);
                            
                        } catch(const out_of_range& e) {
                            continue;
                        }
                    } else {n_voxel = grid.at(n_coord.x).at(n_coord.y).at(n_coord.z);}

                    if(n_voxel != VOXEL::EMPTY && n_voxel != VOXEL::AIR) continue;
                
                    GLuint idxStart = vertices.size();

                    vec3 w_coord = ((vec3(origin.x, 0, origin.y) * (float)CHUNK_SIZE) + c_coord) * (float)VOXEL_SIZE;
                    vector<vec3> face = faces.at(neighbor);
                    vec3 color = blockType.at(c_voxel);

                    normals.insert(normals.end(), 4, neighbor);
                    vertices.insert(vertices.end(), {w_coord + (face[0] * halfVoxelSize), w_coord + (face[1] * halfVoxelSize), w_coord + (face[2] * halfVoxelSize), w_coord + (face[3] * halfVoxelSize)});
                    colors.insert(colors.end(), 4, color);
                    indices.insert(indices.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 0, idxStart + 3, idxStart + 1});    
                }

            }
        }
    }

    mesh.vertices.assign(vertices.begin(), vertices.end());
    mesh.colors.assign(colors.begin(), colors.end());
    mesh.normals.assign(normals.begin(), normals.end());
    mesh.indices.assign(indices.begin(), indices.end());
}

bool Chunk::outOfBounds(vec3 coord){
    Box3 chunkBox(vec3(0), vec3(CHUNK_SIZE - 1, worldHeight - 1, CHUNK_SIZE - 1));
    return !chunkBox.isInside(coord);
}

World::World(int worldHeight, int worldDimension, int renderDist, std::string seed, vec2 playerPos) : worldHeight(worldHeight), worldDimension(worldDimension), renderDist(renderDist), lastPlayerPos(playerPos){
    unsigned int seed1; unsigned int seed2; unsigned int seed3;
    int len = seed.length() / 3;

    seed1 = hash(seed.substr(0, len));
    seed2 = hash(seed.substr(len, len));
    seed3 = hash(seed.substr(len*2, len));

    maps.heightMap.reseed(seed1);
    maps.moistureMap.reseed(seed2);
    maps.tempMap.reseed(seed3);

    reqThread = thread(&World::requestManager, this);

    lock_guard<mutex> reqQueueLock(requestQueueMutex);
    for(int x = 0; x < renderDist; x++){
            for(int y = 0; y < renderDist; y++){
                cout << roundf(((float)((x * renderDist) + y) / (float)(renderDist * renderDist)) * 100.0f) << "% generating: (" << x << ", " << y << ")\n";
                requestQueue.push({REQUEST::CHUNKCREATION, vec2(x, y)});
            }
    }

    requestQueueCV.notify_one();
}

World::~World(){
    running = false;
    requestQueueCV.notify_all();
    reqThread.detach();
}

void World::update(vec3 playerPos, float totalElapsedTime){
    vec2 playerChunkPos = floor(playerPos.xz() / ((float)VOXEL_SIZE * (float)CHUNK_SIZE));
    vec2 difference = playerChunkPos - lastPlayerPos;
    bvec2 moved(greaterThanEqual(abs(difference), vec2(3.0f)));
    if((int)floor(totalElapsedTime) % 10 == 0) lastPlayerPos = playerChunkPos;
    if(any(moved) || edited){
        renderBox.min = round(playerChunkPos - (renderDist * 0.5f));
        renderBox.max = round(playerChunkPos + (renderDist * 0.5f));
        lock_guard<mutex> worldLock(worldMutex);
        for(int x = renderBox.min.x; x < renderBox.max.x; x++){
            for(int y = renderBox.min.y; y < renderBox.max.y; y++){
                vec2 c_chunk(x, y);
                try{
                    auto& chunk = world.at(c_chunk);
                    if(!chunk.dirty) {
                        if(chunk.loaded) continue;
                        lock_guard<mutex> requestQueueLock(requestQueueMutex);
                        requestQueue.push({REQUEST::LOAD, c_chunk});
                        requestQueueCV.notify_one();
                        continue;
                    }
                    
                    lock_guard<mutex> requestQueueLock(requestQueueMutex);
                    requestQueue.push({REQUEST::GENMESH, c_chunk});
                    requestQueueCV.notify_one();

                } catch(const out_of_range& e){
                    lock_guard<mutex> requestQueueLock(requestQueueMutex);
                    requestQueue.push({REQUEST::CHUNKCREATION, c_chunk});
                    requestQueueCV.notify_one();
                }

            }
        }

        for(const auto& it : world){
            if(renderBox.isInside(it.second.origin)) continue;
            lock_guard<mutex> requestQueueLock(requestQueueMutex);
            requestQueue.push({REQUEST::UNLOAD, it.second.origin});
            requestQueueCV.notify_one();
        }
    }
}

void World::sendBuffers(){
    lock_guard<mutex> renderableLock(renderableMutex);
    if(renderable.empty()) return;
    cout << renderable.size() << "\n";

    for(auto chunk : renderable){
        if(chunk->buffered) continue;

        if(!chunk->buffersGenerated){
            glGenVertexArrays(1, &chunk->vao);
            glGenBuffers(1, &chunk->vbo);
            glGenBuffers(1, &chunk->ebo);
            chunk->buffersGenerated = true;
        }

        glBindVertexArray(chunk->vao);
        glBindBuffer(GL_ARRAY_BUFFER, chunk->vbo);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, chunk->ebo);

        glBufferData(GL_ELEMENT_ARRAY_BUFFER, chunk->mesh.indices.size() * sizeof(unsigned int), chunk->mesh.indices.data(), GL_DYNAMIC_DRAW);

        glBufferData(GL_ARRAY_BUFFER, (chunk->mesh.vertices.size() + chunk->mesh.normals.size() + chunk->mesh.colors.size()) * sizeof(vec3), nullptr, GL_DYNAMIC_DRAW);

        glBufferSubData(GL_ARRAY_BUFFER, 0, chunk->mesh.vertices.size() * sizeof(vec3), chunk->mesh.vertices.data());
        glBufferSubData(GL_ARRAY_BUFFER, (chunk->mesh.vertices.size()) * sizeof(vec3), chunk->mesh.normals.size() * sizeof(vec3), chunk->mesh.normals.data());
        glBufferSubData(GL_ARRAY_BUFFER, (chunk->mesh.vertices.size() + chunk->mesh.normals.size()) * sizeof(vec3), chunk->mesh.colors.size() * sizeof(vec3), chunk->mesh.colors.data());
    
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)((chunk->mesh.vertices.size()) * sizeof(vec3)));
        glEnableVertexAttribArray(1);

        glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)((chunk->mesh.vertices.size() + chunk->mesh.normals.size()) * sizeof(vec3)));
        glEnableVertexAttribArray(2);

        glBindVertexArray(0);
        chunk->buffered = true;
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, vec3 playerPos, Shader program){
    glEnable(GL_DEPTH_TEST);

    program.use();
    lock_guard<mutex> renderableLock(renderableMutex);
    if(renderable.empty()) return;

    for(auto& chunk : renderable){
        program.setUniform("model", model);
        program.setUniform("view", view);
        program.setUniform("projection", projection);
        program.setUniform("lightPosition", vec3(0, 30, 0));
        program.setUniform("viewPos", playerPos);

        glBindVertexArray(chunk->vao);
        glDrawElements(GL_TRIANGLES, chunk->mesh.indices.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

void World::requestManager(){
    while(running){
        unique_lock<mutex> lock(requestQueueMutex);
        requestQueueCV.wait(lock, [this] {return !requestQueue.empty() || !running;});
        if(!running) break;

        int maxIter = requestsPerFrame;
        if(requestQueue.size() < requestsPerFrame){
            maxIter = requestQueue.size();
        }

        for(int iter = 0; iter < maxIter; iter++){
            if(requestQueue.empty()) continue;
            auto request = requestQueue.front();
            requestQueue.pop();
            lock.unlock();
            switch(request.first){
                case REQUEST::GENMESH: {
                    lock_guard<mutex> worldLock(worldMutex);
                    Chunk& chunk = world.at(request.second);
                    chunk.genMesh(world);
                    chunk.dirty = false;
                    break;
                }

                case REQUEST::CHUNKCREATION: {
                    lock_guard<mutex> worldLock(worldMutex);
                    Chunk newChunk(maps, request.second, worldHeight);
                    newChunk.genMesh(world);
                    newChunk.dirty = false;
                    newChunk.loaded = true;
                    world.insert({request.second, newChunk});
                    lock_guard<mutex> renderableLock(renderableMutex);
                    renderable.push_back(&world.at(request.second));
                    break;
                }

                case REQUEST::LOAD: {
                    lock_guard<mutex> worldLock(worldMutex);
                    Chunk& chunk = world.at(request.second);
                    chunk.loaded = true;
                    lock_guard<mutex> renderableLock(renderableMutex);
                    renderable.push_back(&chunk);
                    break;
                }

                case REQUEST::UNLOAD: {
                    lock_guard renderableLock(renderableMutex);
                    vec2 origin = request.second;
                    renderable.erase(remove_if(renderable.begin(), renderable.end(), [origin](Chunk* chunk){ chunk->loaded = false; return chunk->origin == origin;}), renderable.end());

                    break;
                }
            }
            lock.lock();
        }
    }
}

int World::hash(std::string string){
    int hash = 0;
    for(auto c : string){
        hash += (int)c;
    }

    return hash;
}   