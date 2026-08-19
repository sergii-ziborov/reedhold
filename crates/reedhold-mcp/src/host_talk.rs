//! Talk overlay methods on the MCP host.

use crate::host::Host;
use reedhold_api::{CircleView, RouteView, Session, TalkNet, TalkView};
use reedhold_core::{Error, Result};

impl Host {
    pub(crate) fn talk_open(
        &mut self,
        epoch: u64,
        prior: &str,
        candidates: &[String],
        company: Option<&str>,
    ) -> Result<()> {
        let mut talk = TalkNet::open(epoch, prior, candidates, company, None)?;
        if let Ok(session) = self.session() {
            talk.online(&session.peer_hex())?;
        }
        self.talk = Some(talk);
        Ok(())
    }

    pub(crate) fn talk_online(&mut self, peer: &str) -> Result<()> {
        self.talk_mut()?.online(peer)
    }

    pub(crate) fn talk_offline(&mut self, peer: &str) -> Result<()> {
        self.talk_mut()?.offline(peer)
    }

    pub(crate) fn talk_block(&mut self, peer: &str) -> Result<()> {
        self.talk_mut()?.block(peer)
    }

    pub(crate) fn talk_dm(
        &mut self,
        to: &str,
        to_msg_pub: &str,
        plaintext: &str,
    ) -> Result<RouteView> {
        let (talk, session) = self.talk_and_session()?;
        talk.dm(session, to, to_msg_pub, plaintext)
    }

    pub(crate) fn talk_create_group(&mut self, name: &str) -> Result<CircleView> {
        let (talk, session) = self.talk_and_session()?;
        talk.create_circle(session, name)
    }

    pub(crate) fn talk_invite(
        &mut self,
        group: &str,
        member: &str,
        member_msg_pub: &str,
    ) -> Result<RouteView> {
        let (talk, session) = self.talk_and_session()?;
        talk.invite(session, group, member, member_msg_pub)
    }

    pub(crate) fn talk_send(&mut self, group: &str, plaintext: &str) -> Result<Vec<RouteView>> {
        let (talk, session) = self.talk_and_session()?;
        talk.send_circle(session, group, plaintext)
    }

    pub(crate) fn talk_remove(&mut self, group: &str, member: &str) -> Result<Vec<RouteView>> {
        let (talk, session) = self.talk_and_session()?;
        talk.remove(session, group, member)
    }

    pub(crate) fn talk_inbox(&mut self) -> Result<Vec<TalkView>> {
        let (talk, session) = self.talk_and_session()?;
        talk.inbox(session)
    }

    fn talk_mut(&mut self) -> Result<&mut TalkNet> {
        self.talk
            .as_mut()
            .ok_or(Error::Mesh("talk net is not open"))
    }

    fn talk_and_session(&mut self) -> Result<(&mut TalkNet, &mut Session)> {
        let talk = self
            .talk
            .as_mut()
            .ok_or(Error::Mesh("talk net is not open"))?;
        let session = self
            .session
            .as_mut()
            .ok_or(Error::Identity("no unlocked session"))?;
        Ok((talk, session))
    }
}
