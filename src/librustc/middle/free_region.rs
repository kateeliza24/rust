// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This file handles the relationships between free regions --
//! meaning lifetime parameters. Ordinarily, free regions are
//! unrelated to one another, but they can be related via implied or
//! explicit bounds.  In that case, we track the bounds using the
//! `TransitiveRelation` type and use that to decide when one free
//! region outlives another and so forth.

use infer::outlives::free_region_map::{FreeRegionMap, FreeRegionRelations};
use hir::def_id::DefId;
use middle::region;
use ty::{self, TyCtxt, Region};

/// Combines a `region::ScopeTree` (which governs relationships between
/// scopes) and a `FreeRegionMap` (which governs relationships between
/// free regions) to yield a complete relation between concrete
/// regions.
///
/// This stuff is a bit convoluted and should be refactored, but as we
/// move to NLL it'll all go away anyhow.
pub struct RegionRelations<'a, 'gcx: 'tcx, 'tcx: 'a> {
    pub tcx: TyCtxt<'a, 'gcx, 'tcx>,

    /// context used to fetch the region maps
    pub context: DefId,

    /// region maps for the given context
    pub region_scope_tree: &'a region::ScopeTree,

    /// free-region relationships
    pub free_regions: &'a FreeRegionMap<'tcx>,
}

impl<'a, 'gcx, 'tcx> RegionRelations<'a, 'gcx, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'a, 'gcx, 'tcx>,
        context: DefId,
        region_scope_tree: &'a region::ScopeTree,
        free_regions: &'a FreeRegionMap<'tcx>,
    ) -> Self {
        Self {
            tcx,
            context,
            region_scope_tree,
            free_regions,
        }
    }

    /// Determines whether one region is a subregion of another.  This is intended to run *after
    /// inference* and sadly the logic is somewhat duplicated with the code in infer.rs.
    pub fn is_subregion_of(&self,
                           sub_region: ty::Region<'tcx>,
                           super_region: ty::Region<'tcx>)
                           -> bool {
        let result = sub_region == super_region || {
            match (sub_region, super_region) {
                (ty::ReEmpty, _) |
                (_, ty::ReStatic) =>
                    true,

                (ty::ReScope(sub_scope), ty::ReScope(super_scope)) =>
                    self.region_scope_tree.is_subscope_of(*sub_scope, *super_scope),

                (ty::ReScope(sub_scope), ty::ReEarlyBound(ref br)) => {
                    let fr_scope = self.region_scope_tree.early_free_scope(self.tcx, br);
                    self.region_scope_tree.is_subscope_of(*sub_scope, fr_scope)
                }

                (ty::ReScope(sub_scope), ty::ReFree(fr)) => {
                    let fr_scope = self.region_scope_tree.free_scope(self.tcx, fr);
                    self.region_scope_tree.is_subscope_of(*sub_scope, fr_scope)
                }

                (ty::ReEarlyBound(_), ty::ReEarlyBound(_)) |
                (ty::ReFree(_), ty::ReEarlyBound(_)) |
                (ty::ReEarlyBound(_), ty::ReFree(_)) |
                (ty::ReFree(_), ty::ReFree(_)) =>
                    self.free_regions.sub_free_regions(sub_region, super_region),

                _ =>
                    false,
            }
        };
        let result = result || self.is_static(super_region);
        debug!("is_subregion_of(sub_region={:?}, super_region={:?}) = {:?}",
               sub_region, super_region, result);
        result
    }

    /// Determines whether this free-region is required to be 'static
    fn is_static(&self, super_region: ty::Region<'tcx>) -> bool {
        debug!("is_static(super_region={:?})", super_region);
        match *super_region {
            ty::ReStatic => true,
            ty::ReEarlyBound(_) | ty::ReFree(_) => {
                let re_static = self.tcx.mk_region(ty::ReStatic);
                self.free_regions.sub_free_regions(&re_static, &super_region)
            }
            _ => false
        }
    }

    pub fn lub_free_regions(&self,
                            r_a: Region<'tcx>,
                            r_b: Region<'tcx>)
                            -> Region<'tcx> {
        self.free_regions.lub_free_regions(self.tcx, r_a, r_b)
    }
}

