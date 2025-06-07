#include "main.hpp"

World::World(float voxelSize, int worldDimension, int worldHeight, int renderDist, std::string seed) : worldHeight(worldHeight), voxelSize(voxelSize), renderDist(renderDist) {
    initialize();

    world = FloatGrid::create(-1.0f);
    world->setTransform(math::Transform::createLinearTransform(voxelSize));

    FloatGrid::Accessor accessor = world->getAccessor();

    unsigned int seed1, seed2, seed3;
    int len = seed.length() / 3;
    char buffer[len];

    size_t end = seed.copy(buffer, len, (len * 0));
    buffer[end] = '\0';
    seed1 = hash((std::string)buffer);

    seed.copy(buffer, len, (len * 1));
    buffer[end] = '\0';
    seed2 = hash((std::string)buffer);

    seed.copy(buffer, len, (len * 2));
    buffer[end] = '\0';
    seed3 = hash((std::string)buffer);

    const PerlinNoise heightMap(seed1);
    const PerlinNoise tempMap(seed2);
    const PerlinNoise moistureMap(seed3); 

    Coord start(-worldDimension/2, 0, -worldDimension/2);

    float _blockType = 1.0f;
    for(int x = 0; x < worldDimension; x++){
        for(int z = 0; z < worldDimension; z++){
            Coord voxel = start.offsetBy(x, 0, z);

            int h = worldHeight * heightMap.octave2D_01(x * 0.01, z * 0.01, 4);

            float temp = tempMap.octave2D_11(x * 0.005, z * 0.005, 4);
            float moistness = moistureMap.octave2D_01(x * 0.005, z * 0.005, 4);
            
            if((temp >= 0.3f && temp <= 1.0f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = 1.0f;}        //hd
            else if ((temp >= 0.3f && temp <= 1.0f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = 2.0f;}    //hn
            else if ((temp >= 0.3f && temp <= 1.0f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = 3.0f;}  //hw
            else if ((temp > -0.3f && temp < 0.3f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = 4.0f;}    //nd
            else if ((temp > -0.3f && temp < 0.3f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = 5.0f;}      //nn
            else if ((temp > -0.3f && temp < 0.3f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = 6.0f;}    //nw
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness >= 0.0f && moistness <= 0.4f)) {_blockType = 7.0f;}  //cd
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness > 0.4f && moistness < 0.7f)) {_blockType = 8.0f;}    //cn
            else if ((temp >= -1.0f && temp <= -0.3f) && (moistness >= 0.7f && moistness <= 1.0f)) {_blockType = 9.0f;}  //cw
            
            for (int y = 0; y < h; y++){
                Coord v = voxel.offsetBy(0, y, 0);
                accessor.setValueOnly(v, _blockType);
            }
        }
    }

    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos){
    glEnable(GL_DEPTH_TEST);
    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("lightPosition", vec3(0, 5, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(vao);
    glDrawElements(GL_TRIANGLES, inds.size(), GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(){
    genRegionMesh();
    setupMeshBuffers();
}

void World::sync(vec3 playerPos){
        Coord player(playerPos.x / voxelSize, playerPos.y / voxelSize, playerPos.z / voxelSize);
        Coord min(player - Coord(renderDist/2)), max(player + Coord(renderDist/2));

        CoordBBox bbox(min, max);

        auto& tree = world->tree();
        tree::LeafManager<FloatGrid::TreeType> leafMgr(tree);

        leafMgr.foreach([&](FloatGrid::TreeType::LeafNodeType& leaf, size_t){
            CoordBBox leafBBox = leaf.getNodeBoundingBox();
            if(!leafBBox.hasOverlap(bbox)){
                for(auto iter = leaf.beginValueOn(); iter; ++iter){
                    leaf.setActiveState(iter.getCoord(), false);
                }
                return;
            };

            CoordBBox _intersect = leafBBox;
            _intersect.intersect(bbox);

            for(CoordBBox::Iterator iter = _intersect.begin(); iter != _intersect.end(); ++iter){
                float val = leaf.getValue(*iter);
                if(val != -1.0f){
                    leaf.setActiveState(*iter, true);
                } else {
                    leaf.setActiveState(*iter, false);
                }
            }
        });
}

World::Mesh World::isSurfaceVoxel(Coord coord, FloatGrid::Accessor accessor){
    Mesh mesh;
    float value = accessor.getValue(coord);

    vector<Coord> directions = {Coord(0, 0, 1),
                                Coord(0, 1, 0),
                                Coord(1, 0, 0),
                                Coord(0, 0, -1),
                                Coord(0, -1, 0),
                                Coord(-1, 0, 0)};

    for(auto direction : directions){
        Coord offset = coord.offsetBy(direction.x(), direction.y(), direction.z());

        if(accessor.getValue(offset) == -1){
            mesh.isSurface == true;
            mesh.directions.push_back(direction);
        }
    }
    

    return mesh;
}

vec3 World::voxelToWorld(Coord coord, float voxelSize){
    Vec3i _coord = coord.asVec3i();

    return vec3(_coord.x() * voxelSize, _coord.y() * voxelSize, _coord.z() * voxelSize);
}

void World::genRegionMesh(){
    FloatGrid::Accessor accessor = world->getAccessor();
    vector<vec3> _verts, _norms, _colors;
    vector<unsigned int> _inds;

    for(auto iter = world->beginValueOn(); iter; ++iter){
        Coord voxel = iter.getCoord();
        Mesh mesh = isSurfaceVoxel(voxel, accessor);

        vec3 voxelCenter = voxelToWorld(voxel, voxelSize);
        if(mesh.isSurface){
            for(auto direction : mesh.directions){
                GLuint idxStart = _verts.size();
                vec3 normal = vec3(direction.asVec3s().x(), direction.asVec3s().y(), direction.asVec3s().z());
                vector<vec3> face = faces[direction];
                vec3 color = blockType[iter.getValue()]; 

                _norms.insert(_norms.end(), {normal, normal, normal, normal});                
                _verts.insert(_verts.end(), {voxelCenter + (face[0] * (voxelSize / 2)), voxelCenter +  (face[1] * (voxelSize / 2)), voxelCenter +  (face[2] * (voxelSize / 2)), voxelCenter +  (face[3] * (voxelSize / 2))});
                _inds.insert(_inds.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 0, idxStart + 3, idxStart + 1});

                _colors.insert(_colors.end(), {color, color, color, color});
            }   
        }
    }

    verts.assign(_verts.begin(), _verts.end());
    inds.assign(_inds.begin(), _inds.end());
    normals.assign(_norms.begin(), _norms.end());
    colors.assign(_colors.begin(), _colors.end());
}

void World::setupMeshBuffers(){
    glBindVertexArray(vao);

    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, (verts.size() + normals.size() + colors.size()) * sizeof(vec3), nullptr, GL_DYNAMIC_DRAW);

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
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, inds.size() * sizeof(unsigned int), inds.data(), GL_DYNAMIC_DRAW);

    glBindVertexArray(0);
}

int World::hash(std::string str){
    int hash = 0;
    for(auto c : str){
        hash += (int)c;
    }
    return hash;
}
