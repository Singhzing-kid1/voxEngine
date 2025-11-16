#include "main.hpp"

Chunk::Chunk(noiseMaps maps, vec2 origin, int worldHeight) : maps(maps), origin(origin), worldHeight(worldHeight), grid(CHUNK_SIZE, vector<vector<VOXEL>>(MAX_WORLD_GENERATION_HEIGHT, vector<VOXEL>(CHUNK_SIZE, VOXEL::EMPTY))){
    for(int x = 0; x < (int)CHUNK_SIZE; x++){
        for(int z = 0; z < (int)CHUNK_SIZE; z++){
            int height = (int)MAX_WORLD_GENERATION_HEIGHT * maps.heightMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * heightMapScalar, ((origin.y * CHUNK_SIZE) + z) * heightMapScalar, perlinNoiseOctaveAmount);
            float moistness = maps.moistureMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.y * CHUNK_SIZE) + z) * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);
            float temp = maps.tempMap.octave2D_11(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.y * CHUNK_SIZE) + z) * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);

            VOXEL _blockType = VOXEL::EMPTY;

            MOISTCAT moisture = categorizeMoisture(moistness);
            TEMPCAT temperature = categorizeTemp(temp);

            _blockType = blockTypeLookup[temperature][moisture];
            
            for(int y = 0; y < MAX_WORLD_GENERATION_HEIGHT; y++){
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
                        vec2 n_chunkCoord = glm::floor(((origin * (float)CHUNK_SIZE) + n_coord.xz()) / (float)CHUNK_SIZE);

                        try{    
                            Chunk& n_chunk = world.at(n_chunkCoord);
                            n_coord = modulo(((vec3(origin.x, 0, origin.y) * (float)CHUNK_SIZE) + n_coord), vec3((float)CHUNK_SIZE, (float)MAX_WORLD_GENERATION_HEIGHT, (float)CHUNK_SIZE));
                            if(neighbor.y != 1){
                                n_voxel = n_chunk.grid.at(n_coord.x).at(n_coord.y).at(n_coord.z);
                            }

                            n_voxel = VOXEL::EMPTY;

                        } catch(const out_of_range& e) {
                            n_coord = modulo(((vec3(origin.x, 0, origin.y) * (float)CHUNK_SIZE) + n_coord), vec3((float)CHUNK_SIZE, (float)MAX_WORLD_GENERATION_HEIGHT, (float)CHUNK_SIZE));
                            vec3 n_worldCoord = ((vec3(n_chunkCoord.x, 0, n_chunkCoord.y) * (float)CHUNK_SIZE) + n_coord);

                            int height = (int)MAX_WORLD_GENERATION_HEIGHT * maps.heightMap.octave2D_01(n_worldCoord.x * heightMapScalar, n_worldCoord.z * heightMapScalar, perlinNoiseOctaveAmount);

                            n_voxel = VOXEL::NORM;

                            if(c_coord.y > height){
                                n_voxel = VOXEL::EMPTY;
                            }

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

vec3 Chunk::modulo(vec3 dividend, vec3 divisor){
    float x = (int)dividend.x % (int)divisor.x < 0 ? ((int)dividend.x % (int)divisor.x) + divisor.x : (int)dividend.x % (int)divisor.x;
    float y = (int)dividend.y % (int)divisor.y < 0 ? ((int)dividend.y % (int)divisor.y) + divisor.y : (int)dividend.y % (int)divisor.y;
    float z = (int)dividend.z % (int)divisor.z < 0 ? ((int)dividend.z % (int)divisor.z) + divisor.z : (int)dividend.z % (int)divisor.z;

    return vec3(x, y, z);
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

    reqThreadOne = thread(&World::requestManager, this);
}

World::~World(){
    running = false;
    requestQueueCV.notify_all();
    reqThreadOne.detach();
}

void World::update(vec3 playerPos, float totalElapsedTime){
    vec2 playerChunkPos = floor(playerPos.xz() / ((float)VOXEL_SIZE * (float)CHUNK_SIZE));
    vec2 difference = playerChunkPos - lastPlayerPos;

    if(!working.load()){
        requestQueueCV.notify_one();
    }

    bvec2 moved(greaterThanEqual(abs(difference), vec2(3.0f)));

    set<vec2, vec2Less> newChunks;

    set<vec2, vec2Less> keep;
    set<vec2, vec2Less> unload;
    set<vec2, vec2Less> load;

    for(int x = round(playerChunkPos.x - (renderDist * 0.5f)); x <= round(playerChunkPos.x + (renderDist * 0.5f)); x++){
        for(int y = round(playerChunkPos.y - (renderDist * 0.5f)); y <= round(playerChunkPos.y + (renderDist * 0.5f)); y++){
            newChunks.insert(vec2(x, y));
        }
    }
    
    set_intersection(oldChunks.begin(), oldChunks.end(), newChunks.begin(), newChunks.end(), inserter(keep, keep.begin()), vec2Less{});
    set_difference(newChunks.begin(), newChunks.end(), keep.begin(), keep.end(), inserter(load, load.begin()), vec2Less{});
    set_difference(oldChunks.begin(), oldChunks.end(), keep.begin(), keep.end(), inserter(unload, unload.begin()), vec2Less{});

    {
        lock_guard<mutex> lock(queueMutex);
        for(const auto& coord : load){
            requestQueue.push({REQUEST::LOAD, coord});
            requestQueueCV.notify_one();
        }

        for(const auto& coord : unload){
            requestQueue.push({REQUEST::UNLOAD, coord});
            requestQueueCV.notify_one();
        }
    }

    oldChunks = newChunks;

}

void World::sendBuffers() {
    lock_guard<mutex> renderLock(renderableMutex);
    auto& renderable = renderableBuffers[currentRenderableIndex.load()];

    if (renderable.empty()) return;

    for (auto& chunk : renderable) {
        if (chunk.second.buffered) continue;

        if (!chunk.second.genBuffers) {
            glGenVertexArrays(1, &chunk.second.vao);
            glGenBuffers(1, &chunk.second.vbo);
            glGenBuffers(1, &chunk.second.ebo);
            chunk.second.genBuffers = true;
        }

        glBindVertexArray(chunk.second.vao);
        glBindBuffer(GL_ARRAY_BUFFER, chunk.second.vbo);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, chunk.second.ebo);

        glBufferData(GL_ELEMENT_ARRAY_BUFFER, chunk.second.mesh.indices.size() * sizeof(unsigned int), chunk.second.mesh.indices.data(), GL_DYNAMIC_DRAW);

        glBufferData(GL_ARRAY_BUFFER, (chunk.second.mesh.vertices.size() + chunk.second.mesh.normals.size() + chunk.second.mesh.colors.size()) * sizeof(vec3), nullptr, GL_DYNAMIC_DRAW);

        glBufferSubData(GL_ARRAY_BUFFER, 0, chunk.second.mesh.vertices.size() * sizeof(vec3), chunk.second.mesh.vertices.data());
        glBufferSubData(GL_ARRAY_BUFFER, chunk.second.mesh.vertices.size() * sizeof(vec3), chunk.second.mesh.normals.size() * sizeof(vec3), chunk.second.mesh.normals.data());
        glBufferSubData(GL_ARRAY_BUFFER, (chunk.second.mesh.vertices.size() + chunk.second.mesh.normals.size()) * sizeof(vec3), chunk.second.mesh.colors.size() * sizeof(vec3), chunk.second.mesh.colors.data());

        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)0);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)(chunk.second.mesh.vertices.size() * sizeof(vec3)));
        glEnableVertexAttribArray(1);

        glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*)((chunk.second.mesh.vertices.size() + chunk.second.mesh.normals.size()) * sizeof(vec3)));
        glEnableVertexAttribArray(2);

        glBindVertexArray(0);

        chunk.second.buffered = true;
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, vec3 playerPos, Shader program) {
    const auto& renderable = renderableBuffers[currentRenderableIndex.load()];

    glEnable(GL_DEPTH_TEST);
    program.use();

    for (const auto& chunk : renderable) {
        program.setUniform("model", model);
        program.setUniform("view", view);
        program.setUniform("projection", projection);
        program.setUniform("lightPosition", vec3(0, 30, 0));
        program.setUniform("viewPos", playerPos);

        glBindVertexArray(chunk.second.vao);
        glDrawElements(GL_TRIANGLES, chunk.second.mesh.indices.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

void World::requestManager() {
    vector<pair<REQUEST, vec2>> requestsToProcess;
    requestsToProcess.reserve(requestsPerFrame);

    while (running.load()) {
        {
            unique_lock<mutex> lock(queueMutex);
            working = false;
            requestQueueCV.wait(lock, [this] { return !requestQueue.empty() || !running.load(); });
            if (!running.load()) break;

            working = true;

            requestsToProcess.clear();
            for (int i = 0; i < requestsPerFrame && !requestQueue.empty(); ++i) {
                requestsToProcess.push_back(requestQueue.front());
                requestQueue.pop();
            }
        }

        vector<pair<REQUEST, vec2>> deferredNewRequests;
        deferredNewRequests.reserve(requestsPerFrame);

        size_t backBufferIndex = 1 - currentRenderableIndex.load();
        {
            lock_guard<mutex> renderLock(renderableMutex);
            renderableBuffers[backBufferIndex] = renderableBuffers[currentRenderableIndex.load()];
        }

        for (const auto& request : requestsToProcess) {
            switch (request.first) {
                case REQUEST::CHUNKCREATION: {
                    {
                        lock_guard<mutex> lock(worldMutex);
                        world.emplace(request.second, Chunk(maps, request.second, worldHeight));
                    }
                    deferredNewRequests.push_back({REQUEST::GENMESH, request.second});
                    break;
                }
                case REQUEST::GENMESH: {
                    Chunk chunkCopy;
                    {
                        lock_guard<mutex> lock(worldMutex);
                        chunkCopy = world.at(request.second);
                    }

                    chunkCopy.genMesh(world);
                    chunkCopy.dirty = false;

                    {
                        lock_guard<mutex> lock(worldMutex);
                        world[request.second] = chunkCopy;
                    }

                    if (!chunkCopy.loaded) {
                        deferredNewRequests.push_back({REQUEST::LOAD, request.second});
                    } else {
                        deferredNewRequests.push_back({REQUEST::UPDATE, request.second});
                    }
                    break;
                }
                case REQUEST::LOAD: {
                    try {
                        Chunk chunkCopy;
                        {
                            lock_guard<mutex> lock(worldMutex);
                            chunkCopy = world.at(request.second);
                            chunkCopy.loaded = true;
                            world[request.second] = chunkCopy;
                        }

                        {
                            lock_guard<mutex> renderLock(renderableMutex);
                            renderableBuffers[backBufferIndex].push_back({request.second, chunkCopy});
                        }
                    } catch (const out_of_range& e) {
                        deferredNewRequests.push_back({REQUEST::CHUNKCREATION, request.second});
                    }
                    break;
                }
                case REQUEST::UNLOAD: {
                    {
                        std::lock_guard<std::mutex> lock(worldMutex);
                        try{
                            auto& chunk = world.at(request.second);

                            chunk.loaded = false;
                            world[request.second] = chunk;
                        } catch (const std::out_of_range& e){
                            cout << "out of range at: " << to_string(request.second) << "\n";
                        }
                    }

                    {
                        lock_guard<mutex> renderLock(renderableMutex);
                        auto& buffer = renderableBuffers[backBufferIndex];
                        buffer.erase(remove_if(buffer.begin(), buffer.end(),
                            [&request](const pair<vec2, Chunk>& c) { return c.first == request.second; }),
                            buffer.end());
                    }
                    break;
                }
                default:
                    break;
            }
        }

        currentRenderableIndex.store(backBufferIndex);

        if (!deferredNewRequests.empty()) {
            lock_guard<mutex> lock(queueMutex);
            for (auto& req : deferredNewRequests) {
                requestQueue.push(req);
            }
            requestQueueCV.notify_one();
        }
    }
}


unsigned int World::hash(std::string string){
    unsigned int hash = 0;
    for(auto c : string){
        hash += (unsigned int)c;
    }

    return hash;
}