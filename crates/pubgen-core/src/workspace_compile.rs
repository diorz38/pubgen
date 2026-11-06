use crate::plan::{CommandPlan, PublicationOp};
use crate::workspace::{CardKind, CardScope};
use crate::workspace_validation::WorkspaceDraft;

impl WorkspaceDraft {
    pub fn command_plan(&self, content_root: &std::path::Path) -> Result<CommandPlan, Vec<crate::workspace_validation::WorkspaceDiagnostic>> {
        let diagnostics = self.validate();
        if diagnostics.iter().any(|item| item.severity == crate::workspace_validation::ValidationSeverity::Error) { return Err(diagnostics); }
        let mut plan = CommandPlan::new();
        plan.push(PublicationOp::CreateCover { title: self.metadata.title.clone(), subtitle: self.metadata.title_en.clone().unwrap_or_default(), release_no: self.metadata.extra.get("release_no").cloned().unwrap_or_default(), agency: self.metadata.extra.get("agency").cloned().unwrap_or_default() });
        for (index, card) in self.metadata.cards.iter().enumerate() {
            match card.kind {
                CardKind::Cover => {}
                CardKind::Divider => plan.push(PublicationOp::CreateDivider { chapter: index.saturating_add(1) as u32, title: card.title.clone() }),
                CardKind::Toc => plan.push(PublicationOp::GenerateToc { title: card.title.clone(), style: "default".into() }),
                CardKind::Table => push_source(&mut plan, card, content_root, true),
                CardKind::Narrative => push_source(&mut plan, card, content_root, false),
                CardKind::Chart | CardKind::MapChart => plan.push(PublicationOp::InsertChart { spec_json: serde_json::to_string(&card.settings).map_err(|_| diagnostics.clone())? }),
                CardKind::Image => push_source(&mut plan, card, content_root, false),
            }
        }
        Ok(plan)
    }
}

fn push_source(plan: &mut CommandPlan, card: &crate::workspace::PublicationCard, root: &std::path::Path, table: bool) {
    if let Some(source) = &card.source {
        let path = root.join(source);
        if table { plan.push(PublicationOp::InsertTable { source: path, autoflow: card.scope == CardScope::NewObject }); }
        else { plan.push(PublicationOp::InsertStory { source: path, autoflow: card.scope == CardScope::NewObject }); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::excel::IngestReport;
    use crate::workspace::{Language, PageLayout, PublicationCard, PublicationMetadata};
    use std::collections::BTreeMap;
    #[test]
    fn builds_plan_for_valid_workspace() {
        let metadata = PublicationMetadata { profile_id: "bps_brs".into(), layout: PageLayout::Double, language: Language::Id, title: "T".into(), title_en: None, cards: vec![PublicationCard { id: "t1".into(), kind: CardKind::Table, scope: CardScope::NewPage, source: Some("T1.csv".into()), title: "Table".into(), settings: BTreeMap::new() }], extra: BTreeMap::new() };
        let draft = WorkspaceDraft::from_ingest("x".into(), IngestReport { profile_id: "bps_brs".into(), metadata, csv: vec![("T1.csv".into(), b"x".to_vec())], warnings: vec![] });
        let plan = draft.command_plan(std::path::Path::new("work")).expect("plan");
        assert_eq!(plan.len(), 2);
    }
}
