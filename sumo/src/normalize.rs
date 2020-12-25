//! Transforms a `raw::Network` into a `Network` that's easier to reason about.

use std::collections::BTreeMap;
use std::error::Error;

use abstutil::Timer;
use geom::{Distance, PolyLine, Pt2D, Ring};

use crate::{raw, Edge, Junction, Lane, Network};

impl Network {
    /// Reads a .net.xml file and return the normalized SUMO network.
    pub fn load(path: &str, timer: &mut Timer) -> Result<Network, Box<dyn Error>> {
        let raw = raw::Network::parse(path, timer)?;
        timer.start("normalize");
        let network = Network::from_raw(raw);
        timer.stop("normalize");
        Ok(network)
    }

    fn from_raw(raw: raw::Network) -> Network {
        let mut network = Network {
            location: raw.location,
            edges: BTreeMap::new(),
            junctions: BTreeMap::new(),
        };

        let types: BTreeMap<String, raw::Type> =
            raw.types.into_iter().map(|t| (t.id.clone(), t)).collect();

        for junction in raw.junctions {
            if junction.junction_type == "internal" {
                continue;
            }
            network.junctions.insert(
                junction.id.clone(),
                Junction {
                    pt: junction.pt(),
                    id: junction.id,
                    junction_type: junction.junction_type,
                    incoming_lanes: junction.incoming_lanes,
                    internal_lanes: junction.internal_lanes,
                    shape: junction.shape.unwrap(),
                },
            );
        }

        for edge in raw.edges {
            if edge.function == raw::Function::Internal {
                continue;
            }
            let (from, to) = match (edge.from, edge.to) {
                (Some(from), Some(to)) => (from, to),
                _ => {
                    continue;
                }
            };
            let template = &types[edge.edge_type.as_ref().unwrap()];

            let raw_center_line = match edge.shape {
                Some(pl) => pl,
                None => {
                    PolyLine::must_new(vec![network.junctions[&from].pt, network.junctions[&to].pt])
                }
            };
            // TODO I tried interpreting the docs and shifting left/right by 1x or 0.5x of the total
            // road width, but the results don't look right.
            let center_line = match edge.spread_type {
                raw::SpreadType::Center => raw_center_line,
                raw::SpreadType::Right => raw_center_line,
                raw::SpreadType::RoadCenter => raw_center_line,
            };

            let mut lanes = Vec::new();
            for lane in edge.lanes {
                lanes.push(Lane {
                    id: lane.id,
                    index: lane.index,
                    speed: lane.speed,
                    length: lane.length,
                    // https://sumo.dlr.de/docs/Simulation/SublaneModel.html
                    width: lane.width.unwrap_or(Distance::meters(3.2)),
                    center_line: lane.shape.unwrap(),
                    allow: lane.allow,
                });
            }

            network.edges.insert(
                edge.id.clone(),
                Edge {
                    id: edge.id,
                    edge_type: edge.edge_type.unwrap(),
                    name: edge.name,
                    from,
                    to,
                    priority: edge.priority.unwrap_or_else(|| template.priority),
                    lanes,
                    center_line,
                },
            );
        }

        network.fix_coordinates();
        network
    }

    fn fix_coordinates(&mut self) {
        // I tried netconvert's --flip-y-axis option, but it makes all of the y coordinates
        // extremely negative.

        let max_y = self.location.converted_boundary.max_y;
        let fix = |pt: &Pt2D| Pt2D::new(pt.x(), max_y - pt.y());

        for junction in self.junctions.values_mut() {
            junction.pt = fix(&junction.pt);
            junction.shape =
                Ring::must_new(junction.shape.points().iter().map(fix).collect()).to_polygon();
        }
        for edge in self.edges.values_mut() {
            edge.center_line =
                PolyLine::must_new(edge.center_line.points().iter().map(fix).collect());
            for lane in &mut edge.lanes {
                lane.center_line =
                    PolyLine::must_new(lane.center_line.points().iter().map(fix).collect());
            }
        }
    }
}
