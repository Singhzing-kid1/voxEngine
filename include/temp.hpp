#ifndef TEMP_HPP
#define TEMP_HPP

#include "main.hpp"

class Shader;

struct DebugLine {
    glm::vec3 from;
    glm::vec3 to;
    glm::vec3 color;
};

class DebugDrawer : public btIDebugDraw {
    public:
        DebugDrawer();
        ~DebugDrawer();

        void setMatrices(const glm::mat4& model, const glm::mat4& view, const glm::mat4& proj);
        void render(Shader);

        virtual void drawLine(const btVector3& from, const btVector3& to, const btVector3& color) override;
        virtual void drawLine(const btVector3& from, const btVector3& to, const btVector3& fromColor, const btVector3& toColor) override {
            drawLine(from, to, fromColor);
        }
        virtual void drawContactPoint(const btVector3&, const btVector3&, btScalar, int, const btVector3&) override {}
        virtual void reportErrorWarning(const char*) override {}
        virtual void draw3dText(const btVector3&, const char*) override {}
        virtual void setDebugMode(int mode) override { m_debugMode = mode; }
        virtual int getDebugMode() const override { return m_debugMode; }

    private:
        std::vector<DebugLine> m_lines;
        unsigned int m_vao, m_vbo;
        glm::mat4 m_model, m_view, m_proj;
        int m_debugMode;
};

#endif