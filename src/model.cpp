#include "main.hpp"

Model::Model(const char* modelPath, int size){
    ifstream model(modelPath);
    string line;
    vector<string> data;

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
        int tempI;
        vec3 tempV;
        for(int x = 0; x < 3; x++){
            ss >> tempI;
            tempV[x] = tempI;
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