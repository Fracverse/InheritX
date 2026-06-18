use super::errors::DatabaseError;
use super::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// External genetic database client trait.
#[async_trait]
pub trait GeneticDatabaseClient: Send + Sync {
    async fn query_variant_significance(&self, rsid: &str) -> Result<VariantInfo, DatabaseError>;
    async fn lookup_disease_associations(
        &self,
        variants: &[String],
    ) -> Result<Vec<DiseaseAssociation>, DatabaseError>;
    async fn get_population_frequencies(
        &self,
        variants: &[String],
    ) -> Result<HashMap<String, f64>, DatabaseError>;
}

/// ClinVar database client with embedded reference data.
pub struct ClinVarClient {
    variants: HashMap<String, VariantInfo>,
}

impl Default for ClinVarClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClinVarClient {
    pub fn new() -> Self {
        let variants = HashMap::from([
            (
                "rs429358".into(),
                VariantInfo {
                    rsid: "rs429358".into(),
                    significance: VariantSignificance::LikelyPathogenic,
                    clinical_significance: "Pathogenic/Likely pathogenic".into(),
                    review_status: "reviewed by expert panel".into(),
                },
            ),
            (
                "rs6025".into(),
                VariantInfo {
                    rsid: "rs6025".into(),
                    significance: VariantSignificance::Pathogenic,
                    clinical_significance: "Pathogenic".into(),
                    review_status: "practice guideline".into(),
                },
            ),
            (
                "rs1801133".into(),
                VariantInfo {
                    rsid: "rs1801133".into(),
                    significance: VariantSignificance::LikelyPathogenic,
                    clinical_significance: "Likely pathogenic".into(),
                    review_status: "criteria provided".into(),
                },
            ),
            (
                "rs7903146".into(),
                VariantInfo {
                    rsid: "rs7903146".into(),
                    significance: VariantSignificance::Uncertain,
                    clinical_significance: "Risk factor".into(),
                    review_status: "no assertion criteria provided".into(),
                },
            ),
        ]);

        Self { variants }
    }
}

#[async_trait]
impl GeneticDatabaseClient for ClinVarClient {
    async fn query_variant_significance(&self, rsid: &str) -> Result<VariantInfo, DatabaseError> {
        self.variants
            .get(rsid)
            .cloned()
            .ok_or_else(|| DatabaseError::VariantNotFound(rsid.to_string()))
    }

    async fn lookup_disease_associations(
        &self,
        variants: &[String],
    ) -> Result<Vec<DiseaseAssociation>, DatabaseError> {
        let known: HashMap<&str, (&str, f64)> = HashMap::from([
            ("rs429358", ("Alzheimer's Disease", 3.5)),
            ("rs6025", ("Factor V Leiden Thrombophilia", 5.0)),
            ("rs7903146", ("Type 2 Diabetes", 1.4)),
        ]);

        Ok(variants
            .iter()
            .filter_map(|rsid| {
                known
                    .get(rsid.as_str())
                    .map(|(disease, or)| DiseaseAssociation {
                        rsid: rsid.clone(),
                        disease_name: disease.to_string(),
                        odds_ratio: *or,
                        p_value: 1e-8,
                    })
            })
            .collect())
    }

    async fn get_population_frequencies(
        &self,
        variants: &[String],
    ) -> Result<HashMap<String, f64>, DatabaseError> {
        let frequencies: HashMap<&str, f64> = HashMap::from([
            ("rs429358", 0.15),
            ("rs6025", 0.05),
            ("rs1801133", 0.35),
            ("rs7903146", 0.30),
        ]);

        Ok(variants
            .iter()
            .filter_map(|rsid| {
                frequencies
                    .get(rsid.as_str())
                    .map(|freq| (rsid.clone(), *freq))
            })
            .collect())
    }
}

/// dbSNP reference database client.
pub struct DbSnpClient {
    allele_frequencies: HashMap<String, f64>,
}

impl Default for DbSnpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DbSnpClient {
    pub fn new() -> Self {
        Self {
            allele_frequencies: HashMap::from([
                ("rs429358".into(), 0.15),
                ("rs7412".into(), 0.08),
                ("rs1801133".into(), 0.35),
                ("rs6025".into(), 0.05),
                ("rs7903146".into(), 0.30),
                ("rs1333049".into(), 0.48),
            ]),
        }
    }
}

#[async_trait]
impl GeneticDatabaseClient for DbSnpClient {
    async fn query_variant_significance(&self, rsid: &str) -> Result<VariantInfo, DatabaseError> {
        if self.allele_frequencies.contains_key(rsid) {
            Ok(VariantInfo {
                rsid: rsid.to_string(),
                significance: VariantSignificance::Uncertain,
                clinical_significance: "Variant".into(),
                review_status: "dbSNP reference".into(),
            })
        } else {
            Err(DatabaseError::VariantNotFound(rsid.to_string()))
        }
    }

    async fn lookup_disease_associations(
        &self,
        _variants: &[String],
    ) -> Result<Vec<DiseaseAssociation>, DatabaseError> {
        Ok(Vec::new())
    }

    async fn get_population_frequencies(
        &self,
        variants: &[String],
    ) -> Result<HashMap<String, f64>, DatabaseError> {
        Ok(variants
            .iter()
            .filter_map(|rsid| {
                self.allele_frequencies
                    .get(rsid)
                    .map(|freq| (rsid.clone(), *freq))
            })
            .collect())
    }
}

/// GWAS Catalog database client.
pub struct GwasCatalogClient {
    associations: HashMap<String, Vec<DiseaseAssociation>>,
}

impl Default for GwasCatalogClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GwasCatalogClient {
    pub fn new() -> Self {
        let associations = HashMap::from([
            (
                "rs7903146".into(),
                vec![DiseaseAssociation {
                    rsid: "rs7903146".into(),
                    disease_name: "Type 2 Diabetes".into(),
                    odds_ratio: 1.37,
                    p_value: 1e-50,
                }],
            ),
            (
                "rs1333049".into(),
                vec![DiseaseAssociation {
                    rsid: "rs1333049".into(),
                    disease_name: "Coronary Artery Disease".into(),
                    odds_ratio: 1.29,
                    p_value: 1e-20,
                }],
            ),
        ]);

        Self { associations }
    }
}

#[async_trait]
impl GeneticDatabaseClient for GwasCatalogClient {
    async fn query_variant_significance(&self, rsid: &str) -> Result<VariantInfo, DatabaseError> {
        if self.associations.contains_key(rsid) {
            Ok(VariantInfo {
                rsid: rsid.to_string(),
                significance: VariantSignificance::Uncertain,
                clinical_significance: "Associated".into(),
                review_status: "GWAS significant".into(),
            })
        } else {
            Err(DatabaseError::VariantNotFound(rsid.to_string()))
        }
    }

    async fn lookup_disease_associations(
        &self,
        variants: &[String],
    ) -> Result<Vec<DiseaseAssociation>, DatabaseError> {
        Ok(variants
            .iter()
            .flat_map(|rsid| self.associations.get(rsid).cloned().unwrap_or_default())
            .collect())
    }

    async fn get_population_frequencies(
        &self,
        _variants: &[String],
    ) -> Result<HashMap<String, f64>, DatabaseError> {
        Ok(HashMap::new())
    }
}

/// Composite client that queries multiple genetic databases.
pub struct CompositeGeneticDatabaseClient {
    pub clinvar: ClinVarClient,
    pub dbsnp: DbSnpClient,
    pub gwas: GwasCatalogClient,
}

impl Default for CompositeGeneticDatabaseClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeGeneticDatabaseClient {
    pub fn new() -> Self {
        Self {
            clinvar: ClinVarClient::new(),
            dbsnp: DbSnpClient::new(),
            gwas: GwasCatalogClient::new(),
        }
    }
}

#[async_trait]
impl GeneticDatabaseClient for CompositeGeneticDatabaseClient {
    async fn query_variant_significance(&self, rsid: &str) -> Result<VariantInfo, DatabaseError> {
        // Prefer ClinVar for clinical significance, fall back to dbSNP
        match self.clinvar.query_variant_significance(rsid).await {
            Ok(info) => Ok(info),
            Err(_) => self.dbsnp.query_variant_significance(rsid).await,
        }
    }

    async fn lookup_disease_associations(
        &self,
        variants: &[String],
    ) -> Result<Vec<DiseaseAssociation>, DatabaseError> {
        let mut results = self.clinvar.lookup_disease_associations(variants).await?;
        let gwas_results = self.gwas.lookup_disease_associations(variants).await?;
        results.extend(gwas_results);
        Ok(results)
    }

    async fn get_population_frequencies(
        &self,
        variants: &[String],
    ) -> Result<HashMap<String, f64>, DatabaseError> {
        self.dbsnp.get_population_frequencies(variants).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clinvar_known_variant() {
        let client = ClinVarClient::new();
        let info = client.query_variant_significance("rs429358").await.unwrap();
        assert_eq!(info.significance, VariantSignificance::LikelyPathogenic);
    }

    #[tokio::test]
    async fn test_clinvar_unknown_variant() {
        let client = ClinVarClient::new();
        assert!(client.query_variant_significance("rs999999").await.is_err());
    }

    #[tokio::test]
    async fn test_composite_client_fallback() {
        let client = CompositeGeneticDatabaseClient::new();
        let info = client
            .query_variant_significance("rs1333049")
            .await
            .unwrap();
        assert_eq!(info.rsid, "rs1333049");
    }

    #[tokio::test]
    async fn test_gwas_disease_associations() {
        let client = GwasCatalogClient::new();
        let associations = client
            .lookup_disease_associations(&["rs7903146".into()])
            .await
            .unwrap();
        assert_eq!(associations.len(), 1);
        assert!(associations[0].disease_name.contains("Diabetes"));
    }
}
