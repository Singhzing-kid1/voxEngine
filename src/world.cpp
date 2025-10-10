#include "main.hpp"

Chunk::Chunk(noiseMaps maps, vec2 origin, int worldHeight) : maps(maps), origin(origin), worldHeight(worldHeight), grid(CHUNK_SIZE, vector<vector<VOXEL>>(MAX_WORLD_GENERATION_HEIGHT, vector<VOXEL>(CHUNK_SIZE, VOXEL::EMPTY))){
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

Chunk::~Chunk(){
    (void*)0;
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
        for(int y = 0; y < MAX_WORLD_GENERATION_HEIGHT; y++){
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
    Box3 chunkBox(vec3(0), vec3(CHUNK_SIZE - 1, MAX_WORLD_GENERATION_HEIGHT - 1, CHUNK_SIZE - 1));
    return !chunkBox.isInside(coord);
}

bool Chunk::layerEmpty(int y){
    return all_of(grid.begin(), grid.end(), [y](const auto& layer){
        return all_of(layer.at(y).begin(), layer.at(y).end(), [](VOXEL voxel){
            return voxel == VOXEL::AIR || voxel == VOXEL::EMPTY;
        });
    });
}

bool Chunk::rowEmpty(int x, int y){
    return all_of(grid.at(x).at(y).begin(), grid.at(x).at(y).end(), [](VOXEL voxel){
        return voxel == VOXEL::AIR || voxel == VOXEL::EMPTY;
    });
}


World::World(int worldHeight, int worldDimension, int renderDist, std::string seed, vec2 playerPos) : worldHeight(worldHeight), worldDimension(worldDimension), renderDist(renderDist), lastPlayerPos(playerPos){
    unsigned int seed1; unsigned int seed2; unsigned int seed3;
    int len = seed.length() /3;
    seed1 = hash(seed.substr(0*len, len));
    seed2 = hash(seed.substr(1*len, len));
    seed3 = hash(seed.substr(2*len, len));

    maps.heightMap.reseed(seed1);
    maps.moistureMap.reseed(seed2);
    maps.tempMap.reseed(seed3);

    reqThread = thread(&World::requestManager, this);
}

World::~World(){
    running = false;
    requestQueueCV.notify_one();
    reqThread.detach();
}

void World::update(vec3 playerPos, float totalElapsedTime){
    vec2 playerChunkPos = floor(playerPos.xz() / ((float)VOXEL_SIZE * (float)CHUNK_SIZE));
    vec2 difference = playerChunkPos - lastPlayerPos;

    if(!working.load()){
        requestQueueCV.notify_one();
    }

    bvec2 moved(greaterThanEqual(abs(difference), vec2(3.0f)));

    if(any(moved) || edited || shouldRun){
        shouldRun = false;
        renderBox.min = round(playerChunkPos - (renderDist * 0.5f));
        renderBox.max = round(playerChunkPos + (renderDist * 0.5f));
        for(int x = renderBox.min.x; x < renderBox.max.x; x++){
            for(int y = renderBox.min.y; y < renderBox.max.y; y++){
                vec2 currentChunkCoord(x, y);
                try{
                    scoped_lock lock(worldMutex, queueMutex);
                    auto& chunk = world.at(currentChunkCoord);
                    if(!chunk.dirty) {
                        if(chunk.loaded) continue;
                        requestQueue.push({REQUEST::LOAD, currentChunkCoord});
                        requestQueueCV.notify_one();
                        continue;
                    }

                    requestQueue.push({REQUEST::GENMESH, currentChunkCoord});
                    requestQueueCV.notify_one();
                } catch(const out_of_range& e){
                    lock_guard<mutex> lock(queueMutex);
                    requestQueue.push({REQUEST::CHUNKCREATION, currentChunkCoord});
                    requestQueueCV.notify_one();
                }
            }
        }
        {
            scoped_lock lock(worldMutex, queueMutex);
            for(const auto& it : world){
                if(renderBox.isInside(it.second.origin)) continue;
                requestQueue.push({REQUEST::UNLOAD, it.second.origin});
                requestQueueCV.notify_one();
            }
        }
    }

}

void World::sendBuffers(){
    lock_guard<mutex> lock(renderableMutex);
    if(renderable.empty()) return;

    for(auto& chunk : renderable){
        if(chunk.buffered) continue;

        if(!chunk.genBuffers){
            glGenVertexArrays(1, &chunk.vao);
            glGenBuffers(1, &chunk.vbo);
            glGenBuffers(1, &chunk.ebo);
            chunk.genBuffers = true;
        }

        glBindVertexArray(chunk.vao);
        glBindBuffer(GL_ARRAY_BUFFER, chunk.vbo);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, chunk.ebo);

        glBufferData(GL_ELEMENT_ARRAY_BUFFER, chunk.mesh.indices.size() * sizeof(unsigned int), chunk.mesh.indices.data(), GL_DYNAMIC_DRAW);

        glBufferData(GL_ARRAY_BUFFER, (chunk.mesh.vertices.size() + chunk.mesh.normals.size() + chunk.mesh.colors.size()) * sizeof(vec3), nullptr, GL_DYNAMIC_DRAW);

        glBufferSubData(GL_ARRAY_BUFFER, 0, chunk.mesh.vertices.size() * sizeof(vec3), chunk.mesh.vertices.data());
        glBufferSubData(GL_ARRAY_BUFFER, chunk.mesh.vertices.size() * sizeof(vec3), chunk.mesh.normals.size() * sizeof(vec3), chunk.mesh.normals.data());
        glBufferSubData(GL_ARRAY_BUFFER, (chunk.mesh.vertices.size() + chunk.mesh.normals.size()) * sizeof(vec3), chunk.mesh.colors.size() * sizeof(vec3), chunk.mesh.colors.data());

        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)(chunk.mesh.vertices.size() * sizeof(vec3)));
        glEnableVertexAttribArray(1);

        glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)((chunk.mesh.vertices.size() + chunk.mesh.normals.size()) * sizeof(vec3)));
        glEnableVertexAttribArray(2);

        glBindVertexArray(0);
        chunk.buffered = true;
    }
}

void World::cleanUpRenderable(){
    scoped_lock lock(worldMutex, renderableMutex);
    size_t index;
    auto it = find_if(renderable.begin(), renderable.end(), [](const Chunk& chunk) {return chunk.needsToBeUnloaded;});
    if(it != renderable.end()){
        index = std::distance(renderable.begin(), it);
        auto& chunk = renderable.at(index);
        auto& w_chunk = world.at(chunk.origin);

        w_chunk.needsToBeUnloaded = false;

        renderable.erase(renderable.begin() + index);
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, vec3 playerPos, Shader program){
    lock_guard<mutex> lock(renderableMutex);
    glEnable(GL_DEPTH_TEST);
    program.use();

    for(const auto& chunk : renderable){
        program.setUniform("model", model);
        program.setUniform("view", view);
        program.setUniform("projection", projection);
        program.setUniform("lightPosition", vec3(0, 30, 0));
        program.setUniform("viewPos", playerPos);

        glBindVertexArray(chunk.vao);
        glDrawElements(GL_TRIANGLES, chunk.mesh.indices.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

void World::requestManager(){
    unique_lock<mutex> lock(queueMutex);
    while(running.load()){
        working = false;
        requestQueueCV.wait(lock, [this] {return !requestQueue.empty() || !running.load();});
        if(!running.load()) break;
        working = true;

        vector<pair<REQUEST, vec2>> requestsToProcces;
        requestsToProcces.reserve(requestsPerFrame);

        for(int i = 0; i < requestsPerFrame && !requestQueue.empty(); ++i){
            requestsToProcces.push_back(requestQueue.front());
            requestQueue.pop();
        }

        lock.unlock();

        for(const auto& request : requestsToProcces){
            switch(request.first){
                case REQUEST::CHUNKCREATION: {
                    scoped_lock caseLock(worldMutex, queueMutex);

                    world.emplace(request.second, Chunk(maps, request.second, worldHeight));

                    requestQueue.push({REQUEST::GENMESH, request.second});
                    requestQueueCV.notify_one();
                    break;
                }
                case REQUEST::GENMESH: {
                    scoped_lock caseLock(worldMutex, queueMutex);

                    auto& chunk = world.at(request.second);

                    chunk.genMesh(world);
                    chunk.dirty = false;

                    if(!chunk.loaded){
                        requestQueue.push({REQUEST::LOAD, request.second});
                        requestQueueCV.notify_one();
                        break;
                    }

                    requestQueue.push({REQUEST::UPDATE, request.second});
                    requestQueueCV.notify_one();
                    break;
                }
                case REQUEST::LOAD: {
                    scoped_lock caseLock(worldMutex, renderableMutex);

                    auto& chunk = world.at(request.second);
                    chunk.loaded = true;

                    renderable.push_back(chunk);
                    break;
                }
                case REQUEST::UPDATE: {
                    scoped_lock caseLock(worldMutex, renderableMutex);

                    auto& chunk = world.at(request.second);
                    chunk.buffered = false;

                    size_t index;        
                    auto it = find_if(renderable.begin(), renderable.end(), [request](const Chunk& chunk) {return chunk.origin == request.second;});
                    if(it != renderable.end()){
                        index = std::distance(renderable.begin(), it);
                        renderable[index] = chunk;
                    }
                    break;
                }
                case REQUEST::UNLOAD: {
                    scoped_lock caseLock(worldMutex, renderableMutex);

                    auto& chunk = world.at(request.second);
                    chunk.loaded = false;
                    chunk.needsToBeUnloaded = true;

                    size_t index;
                    auto it = find_if(renderable.begin(), renderable.end(), [request](const Chunk& chunk) {return chunk.origin == request.second;});
                    if(it != renderable.end()){
                        index = std::distance(renderable.begin(), it);
                        renderable[index] = chunk;
                    }
                }
            }
        }
        lock.lock();
    }
}

unsigned int World::hash(std::string string){
    unsigned int hash = 0;
    for(auto c : string){
        hash += (unsigned int)c;
    }

    return hash;
}