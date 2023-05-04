typedef RpsResult (*PFN_CreateCommandResources)(const void *context, void *user_data);

namespace rps {
    class CBackend: public RuntimeBackend {
        public:
            PFN_CreateCommandResources m_cb_CreateCommandResources;
            void* m_user_data;

            CBackend(RenderGraph &renderGraph, PFN_CreateCommandResources cb, void* user_data)
                : RuntimeBackend(renderGraph) {
                m_cb_CreateCommandResources = cb;
                m_user_data = user_data;
            }

            virtual ~CBackend();

            virtual RpsResult RecordCommands(const RenderGraph&                     renderGraph,
                                         const RpsRenderGraphRecordCommandInfo& recordInfo) const override final;

            virtual RpsResult RecordCmdRenderPassBegin(const RuntimeCmdCallbackContext& context) const override final;

            virtual RpsResult RecordCmdRenderPassEnd(const RuntimeCmdCallbackContext& context) const override final;

            virtual RpsResult RecordCmdFixedFunctionBindingsAndDynamicStates(
                const RuntimeCmdCallbackContext& context) const override final;

            virtual void DestroyRuntimeResourceDeferred(ResourceInstance& resource) override final;   

            virtual RpsResult CreateCommandResources(const RenderGraphUpdateContext &context) override final;
        private:
          uint64_t m_heapCounter = 0;     
    };
}
