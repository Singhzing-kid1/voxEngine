#include "main.hpp"

Chunk::Chunk(worldGenInfo genInfo, vec3 origin, int worldHeight) : genInfo(genInfo), origin(origin), grid(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE, VOXEL::EMPTY), worldHeight(worldHeight) {
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int z = 0; z < CHUNK_SIZE; z++){
            int height = worldHeight * genInfo.heightMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * heightMapScalar, ((origin.z * CHUNK_SIZE) + z) * heightMapScalar, perlinNoiseOctaveAmount);
            float temp = genInfo.tempMap.octave2D_11(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.z * CHUNK_SIZE) + z)  * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);
            float moistness = genInfo.moistureMap.octave2D_01(((origin.x * CHUNK_SIZE) + x) * tempAndMoistureMapScalar, ((origin.z * CHUNK_SIZE) + z)  * tempAndMoistureMapScalar, perlinNoiseOctaveAmount);

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

void Chunk::genMesh(const unordered_map<vec3, Chunk, vecHash>& chunks){
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
                            if(chunk.second.origin != _origin) continue;

                            vec3 _voxel = vec3((int)((origin.x * CHUNK_SIZE) + _neighbor.x) % CHUNK_SIZE,
                                               (int)((origin.y * CHUNK_SIZE) + _neighbor.y) % CHUNK_SIZE,   
                                               (int)((origin.z * CHUNK_SIZE) + _neighbor.z) % CHUNK_SIZE);

                                neighborValue = chunk.second.grid.at(_voxel.x + _voxel.y * CHUNK_SIZE + _voxel.z * CHUNK_SIZE * CHUNK_SIZE);
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

World::World(float voxelSize, int worldDimension, int worldHeight, int renderDist, String seed, vec3 playerPos) : worldHeight(worldHeight), worldDimension(worldDimension), renderDist(renderDist) {
    reqThread = thread(&World::requestManager, this);
    bufThread = thread(&World::prepAndCombineMeshes, this);
    unsigned int seed1, seed2, seed3;
    int len = seed.length() / 3;

    seed1 = hash(seed.substr(0, len));
    seed2 = hash(seed.substr(len, len));
    seed3 = hash(seed.substr(len*2, len));

    worldSeed.heightMap.reseed(seed1);
    worldSeed.moistureMap.reseed(seed2);
    worldSeed.tempMap.reseed(seed3);

    world.reserve(renderDist * renderDist);

    renderBox = Box(round((playerPos / (float)CHUNK_SIZE) - (float)renderDist), round((playerPos / (float)CHUNK_SIZE) + (float)renderDist));


/*    for(int x = 0; x < renderDist; x++){
            for(int z = 0; z < renderDist; z++){
                cout << roundf(((float)((x * renderDist) + z) / (float)(renderDist * renderDist)) * 100.0f) << "% generating: (" << x << ", " << z << ")\n";
                lock_guard<mutex> lock(requestQueueMutex);
                requests.push({REQUEST::CHUNKCREATE, vec3(x, 0, z)});
                requests.push({REQUEST::CHUNKCREATE, vec3(x, 1, z)});
                requests.push({REQUEST::CHUNKCREATE, vec3(x, 2, z)});
            }
    } */

    lock_guard<mutex> lock(requestQueueMutex);
    requests.push({REQUEST::CHUNKCREATE, vec3(0, 1, 0)});
    
    requestQueueCV.notify_one();

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);
}

World::~World(){
    running = false;
    requestQueueCV.notify_all();
    if (reqThread.joinable()) reqThread.join();
    if (bufThread.joinable()) bufThread.join();   
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos) {
    lock_guard<mutex> lock(meshMutex);
    Mesh mesh = worldMesh;

    glEnable(GL_DEPTH_TEST);

    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("lightPosition", vec3(0, 30, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(vao);
    glDrawElements(GL_TRIANGLES, mesh.inds.size(), GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(vec3 playerPos) {
    vec3 _playerPos = round(playerPos / (float)CHUNK_SIZE);
    bvec3 moved = greaterThanEqual(abs(_playerPos - lastPlayerPos), vec3(3.0f)); // remove magic numbers
    if(any(moved) || edited || firstTime){
        firstTime = false;
        renderBox.min = round((playerPos / (float)CHUNK_SIZE) - (float)renderDist);
        renderBox.max = round((playerPos / (float)CHUNK_SIZE) + (float)renderDist);
        unordered_map<vec3, Chunk, vecHash> _world;
        {
            lock_guard<mutex> lock(worldMutex);
            _world = world;
        }
        for(int x = renderBox.min.x; x <  renderBox.max.x; x++){
            for(int y = renderBox.min.y; y <  renderBox.max.y; y++){
                for(int z = renderBox.min.z; z <  renderBox.max.z; z++){
                    vec3 coord = vec3(x, y, z);
                    auto it = _world.find(coord);
                    if(it != _world.end()){
                        if(!it->second.dirty) continue;
                        lock_guard<mutex> lock(requestQueueMutex);
                        requests.push({REQUEST::GENERATEMESH, coord});
                        buffered.store(false);
                        continue;
                    }
                    lock_guard<mutex> lock(requestQueueMutex);
                    requests.push({REQUEST::CHUNKCREATE, coord});
                    buffered.store(false);
                }
            }
        }
        requestQueueCV.notify_one();
    }
}

void World::requestManager(){
    while(running){
        unique_lock<mutex> lock(requestQueueMutex);
        requestQueueCV.wait(lock, [this] { return !requests.empty() || !running;});

        if(!running) break;

        cout << "here req\n";

        for(int iter = 0; iter < requestsPerFrame; iter++){
            auto& request = requests.front();
            lock.unlock();
            
            switch(request.first){
                case REQUEST::GENERATEMESH: {
                    lock_guard<mutex> worldLock(worldMutex);
                    Chunk& chunk = world.at(request.second);
                    chunk.genMesh(world);
                    chunk.dirty = false;
                    break;

                }
                case REQUEST::CHUNKCREATE: {
                    Chunk newChunk(worldSeed, request.second, worldHeight);
                    lock_guard<mutex> worldLock(worldMutex);
                    newChunk.genMesh(world);
                    newChunk.dirty = false;
                    world.insert({request.second, newChunk});
                    lock_guard<mutex> renderableLock(renderableMutex);
                    renderable.push_back(&world.at(request.second));
                    break;
                }
            }
            lock.lock();
            requests.pop();
        }

        startMeshing.store(true, memory_order_release);
        startMeshing.notify_one();
    }
}

void World::prepAndCombineMeshes() {
    while(running){
        if(buffered.load()) continue;

        startMeshing.wait(false, memory_order_acquire);
        if(!running) break;
        startMeshing.store(false, memory_order_release);

        size_t lastVertsSize = 0;

        vector<Chunk*> chunksToBuffer;
        {
            lock_guard<mutex> lock(renderableMutex);
            if(renderable.empty()) continue;
            chunksToBuffer = renderable;
        }

        cout << "here mesh\n";
        Mesh _mesh;

        for(const Chunk* chunk : chunksToBuffer){
            Mesh mesh = chunk->mesh;

            _mesh.verts.insert(_mesh.verts.end(), mesh.verts.begin(), mesh.verts.end());
            _mesh.normals.insert(_mesh.normals.end(), mesh.normals.begin(), mesh.normals.end());
            _mesh.colors.insert(_mesh.colors.end(), mesh.colors.begin(), mesh.colors.end());
            for(const auto& ind : mesh.inds){
                _mesh.inds.push_back(ind + lastVertsSize);
            }

            lastVertsSize += mesh.verts.size();
        }

        lock_guard<mutex> meshLock(meshMutex);
        worldMesh = _mesh;
    }
}

void World::setBuffers(){
    if(buffered.load()) return;
    lock_guard<mutex> lock(meshMutex);
    Mesh mesh = worldMesh;

    cout << "here send buffers  " << mesh.verts.size() << "\n";

    glBindVertexArray(vao);

    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, (mesh.verts.size() + mesh.normals.size() + mesh.colors.size()) * sizeof(vec3), nullptr, GL_STATIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, mesh.verts.size() * sizeof(vec3), mesh.verts.data());
    glBufferSubData(GL_ARRAY_BUFFER, mesh.verts.size() * sizeof(vec3), mesh.normals.size() * sizeof(vec3), mesh.normals.data());
    glBufferSubData(GL_ARRAY_BUFFER, (mesh.verts.size() + mesh.normals.size()) * sizeof(vec3), mesh.colors.size() * sizeof(vec3), mesh.colors.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (mesh.verts.size() * sizeof(vec3)));
    glEnableVertexAttribArray(1);

    glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) ((mesh.verts.size() + mesh.normals.size()) * sizeof(vec3)));
    glEnableVertexAttribArray(2);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, mesh.inds.size() * sizeof(unsigned int), mesh.inds.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);

    buffered.store(true);
}

int World::hash(String str){
    int hash = 0;
    for(auto c : str){
        hash += (int)c;
    }

    return hash;
}