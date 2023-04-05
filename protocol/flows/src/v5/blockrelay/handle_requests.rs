use crate::{flow_context::FlowContext, flow_trait::Flow};
use kaspa_core::debug;
use kaspa_p2p_lib::{common::ProtocolError, dequeue, make_message, pb::kaspad_message::Payload, IncomingRoute, Router};
use std::sync::Arc;

pub struct HandleRelayBlockRequests {
    ctx: FlowContext,
    router: Arc<Router>,
    incoming_route: IncomingRoute,
}

#[async_trait::async_trait]
impl Flow for HandleRelayBlockRequests {
    fn name(&self) -> &'static str {
        "HANDLE_RELAY_BLOCK_REQUESTS"
    }

    fn router(&self) -> Option<Arc<Router>> {
        Some(self.router.clone())
    }

    async fn start(&mut self) -> Result<(), ProtocolError> {
        self.start_impl().await
    }
}

impl HandleRelayBlockRequests {
    pub fn new(ctx: FlowContext, router: Arc<Router>, incoming_route: IncomingRoute) -> Self {
        Self { ctx, router, incoming_route }
    }

    async fn start_impl(&mut self) -> Result<(), ProtocolError> {
        loop {
            let msg = dequeue!(self.incoming_route, Payload::RequestRelayBlocks)?;
            let hashes: Vec<_> = msg.try_into()?;

            let consensus = self.ctx.consensus();
            let session = consensus.session().await;

            for hash in hashes {
                let block = session.get_block(hash)?;
                self.router.enqueue(make_message!(Payload::Block, (&block).into())).await?;
                debug!("relayed block with hash {} to peer {}", hash, self.router);
            }
        }
    }
}