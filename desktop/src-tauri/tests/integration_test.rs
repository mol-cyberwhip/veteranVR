use veteran_desktop::services::catalog::CatalogService;

#[test]
fn test_catalog_service_instantiation() {
    let _service = CatalogService::new();
    // Verify we can access public methods
    let hash = CatalogService::game_name_to_hash("Test Game");
    assert_eq!(hash.len(), 32);
}

#[test]
fn test_game_parsing_integration() {
    let content = "Header\nName;Rel;Pkg;1";
    let games = CatalogService::parse_game_list_content(content);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].game_name, "Name");
}
