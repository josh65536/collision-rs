use cgmath::prelude::*;
use cgmath::{BaseFloat, Point3, Vector3};

use crate::prelude::*;
use crate::primitive::util::get_max_point;
use crate::volume::Sphere;
use crate::{Aabb3, Ray3};

/// Cuboid primitive.
///
/// Have a cached set of corner points to speed up computation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cuboid<S> {
    /// Dimensions of the box
    dim: Vector3<S>,
    half_dim: Vector3<S>,
    corners: [Point3<S>; 8],
}

impl<S> Cuboid<S>
where
    S: BaseFloat,
{
    /// Create a new cuboid primitive from component dimensions
    pub fn new(dim_x: S, dim_y: S, dim_z: S) -> Self {
        Self::new_impl(Vector3::new(dim_x, dim_y, dim_z))
    }

    /// Create a new cuboid primitive from a vector of component dimensions
    pub fn new_impl(dim: Vector3<S>) -> Self {
        let half_dim = dim / (S::one() + S::one());
        Self {
            dim,
            half_dim,
            corners: Self::generate_corners(&half_dim),
        }
    }

    /// Get the dimensions of the `Cuboid`
    pub fn dim(&self) -> &Vector3<S> {
        &self.dim
    }

    /// Get the half dimensions of the `Cuboid`
    pub fn half_dim(&self) -> &Vector3<S> {
        &self.half_dim
    }

    fn generate_corners(half_dim: &Vector3<S>) -> [Point3<S>; 8] {
        [
            Point3::new(half_dim.x, half_dim.y, half_dim.z),
            Point3::new(-half_dim.x, half_dim.y, half_dim.z),
            Point3::new(-half_dim.x, -half_dim.y, half_dim.z),
            Point3::new(half_dim.x, -half_dim.y, half_dim.z),
            Point3::new(half_dim.x, half_dim.y, -half_dim.z),
            Point3::new(-half_dim.x, half_dim.y, -half_dim.z),
            Point3::new(-half_dim.x, -half_dim.y, -half_dim.z),
            Point3::new(half_dim.x, -half_dim.y, -half_dim.z),
        ]
    }
}

impl<S> Primitive for Cuboid<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn support_point<T>(&self, direction: &Vector3<S>, transform: &T) -> Point3<S>
    where
        T: Transform<Point3<S>>,
    {
        get_max_point(self.corners.iter(), direction, transform)
    }

    fn closest_valid_normal_local(
        &self,
        normal: &<Self::Point as EuclideanSpace>::Diff,
    ) -> <Self::Point as EuclideanSpace>::Diff {
        if normal.x.abs() > normal.y.abs() && normal.x.abs() > normal.z.abs() {
            Vector3::new(normal.x.signum(), Zero::zero(), Zero::zero())
        } else if normal.y.abs() > normal.z.abs() && normal.y.abs() >= normal.x.abs() {
            Vector3::new(Zero::zero(), normal.y.signum(), Zero::zero())
        } else {
            Vector3::new(Zero::zero(), Zero::zero(), normal.z.signum())
        }
    }
}

impl<S> ComputeBound<Aabb3<S>> for Cuboid<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Aabb3<S> {
        Aabb3::new(
            Point3::from_vec(-self.half_dim),
            Point3::from_vec(self.half_dim),
        )
    }
}

impl<S> ComputeBound<Sphere<S>> for Cuboid<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Sphere<S> {
        let max = self.half_dim.x.max(self.half_dim.y).max(self.half_dim.z);
        Sphere {
            center: Point3::origin(),
            radius: max,
        }
    }
}

impl<S> Discrete<Ray3<S>> for Cuboid<S>
where
    S: BaseFloat,
{
    fn intersects(&self, ray: &Ray3<S>) -> bool {
        Aabb3::new(
            Point3::from_vec(-self.half_dim),
            Point3::from_vec(self.half_dim),
        )
        .intersects(ray)
    }
}

impl<S> Continuous<Ray3<S>> for Cuboid<S>
where
    S: BaseFloat,
{
    type Result = Point3<S>;

    fn intersection(&self, ray: &Ray3<S>) -> Option<Point3<S>> {
        self.intersection_normal(ray).map(|(p, _)| p)
    }
}

impl<S> ContinuousNormal<Ray3<S>> for Cuboid<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn intersection_normal(
        &self,
        ray: &Ray3<S>,
    ) -> Option<(Self::Point, <Self::Point as EuclideanSpace>::Diff)> {
        Aabb3::new(
            Point3::from_vec(-self.half_dim),
            Point3::from_vec(self.half_dim),
        )
        .intersection_normal(ray)
    }
}

/// Cuboid primitive.
///
/// Have a cached set of corner points to speed up computation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cube<S> {
    cuboid: Cuboid<S>,
}

impl<S> Cube<S>
where
    S: BaseFloat,
{
    /// Create a new cube primitive
    pub fn new(dim: S) -> Self {
        Cube {
            cuboid: Cuboid::new(dim, dim, dim),
        }
    }

    /// Get the dimension of the cube
    pub fn dim(&self) -> S {
        self.cuboid.dim.x
    }

    /// Get the half dimension of the cube
    pub fn half_dim(&self) -> S {
        self.cuboid.half_dim.x
    }
}

impl<S> Primitive for Cube<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn support_point<T>(&self, direction: &Vector3<S>, transform: &T) -> Point3<S>
    where
        T: Transform<Point3<S>>,
    {
        self.cuboid.support_point(direction, transform)
    }

    fn closest_valid_normal_local(
        &self,
        normal: &<Self::Point as EuclideanSpace>::Diff,
    ) -> <Self::Point as EuclideanSpace>::Diff {
        self.cuboid.closest_valid_normal_local(normal)
    }
}

impl<S> ComputeBound<Aabb3<S>> for Cube<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Aabb3<S> {
        self.cuboid.compute_bound()
    }
}

impl<S> ComputeBound<Sphere<S>> for Cube<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Sphere<S> {
        self.cuboid.compute_bound()
    }
}

impl<S> Discrete<Ray3<S>> for Cube<S>
where
    S: BaseFloat,
{
    fn intersects(&self, ray: &Ray3<S>) -> bool {
        self.cuboid.intersects(ray)
    }
}

impl<S> Continuous<Ray3<S>> for Cube<S>
where
    S: BaseFloat,
{
    type Result = Point3<S>;

    fn intersection(&self, ray: &Ray3<S>) -> Option<Point3<S>> {
        self.cuboid.intersection(ray)
    }
}

impl<S> ContinuousNormal<Ray3<S>> for Cube<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn intersection_normal(&self, ray: &Ray3<S>) -> Option<(Point3<S>, Vector3<S>)> {
        self.cuboid.intersection_normal(ray)
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_ulps_eq;
    use cgmath::{vec3, Decomposed, Point3, Quaternion, Rad, Vector3};

    use super::*;
    use Ray3;

    #[test]
    fn test_rectangle_bound() {
        let r = Cuboid::new(10., 10., 10.);
        assert_eq!(bound(-5., -5., -5., 5., 5., 5.), r.compute_bound())
    }

    #[test]
    fn test_cuboid_closest_valid_normal() {
        let r = Cuboid::new(1., 1., 1.);

        assert_eq!(
            vec3(1., 0., 0.),
            r.closest_valid_normal_local(&vec3(6.0 / 7.0, -2.0 / 7.0, 3.0 / 7.0))
        );
        assert_eq!(
            vec3(0., 1., 0.),
            r.closest_valid_normal_local(&vec3(3.0 / 7.0, 6.0 / 7.0, -2.0 / 7.0))
        );
        assert_eq!(
            vec3(0., 0., 1.),
            r.closest_valid_normal_local(&vec3(-2.0 / 7.0, 3.0 / 7.0, 6.0 / 7.0))
        );
        assert_eq!(
            vec3(-1., 0., 0.),
            r.closest_valid_normal_local(&vec3(-0.5f64.sqrt(), 0.5, 0.5))
        );
        assert_eq!(
            vec3(0., -1., 0.),
            r.closest_valid_normal_local(&vec3(0.5, -0.5f64.sqrt(), 0.5))
        );
        assert_eq!(
            vec3(0., 0., -1.),
            r.closest_valid_normal_local(&vec3(0.5, 0.5, -0.5f64.sqrt()))
        );
    }

    #[test]
    fn test_ray_discrete() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        assert!(cuboid.intersects(&ray));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert!(!cuboid.intersects(&ray));
    }

    #[test]
    fn test_ray_discrete_transformed() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 1., 0., 0.);
        assert!(cuboid.intersects_transformed(&ray, &t));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert!(!cuboid.intersects_transformed(&ray, &t));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 1., 0., 0.3);
        assert!(cuboid.intersects_transformed(&ray, &t));
    }

    #[test]
    fn test_ray_continuous() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        assert_eq!(Some(Point3::new(5., 0., 0.)), cuboid.intersection(&ray));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(None, cuboid.intersection(&ray));
    }

    #[test]
    fn test_ray_continuous_normal() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(-10., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(
            Some((Point3::new(-5., 0., 0.), vec3(-1., 0., 0.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        assert_eq!(
            Some((Point3::new(5., 0., 0.), vec3(1., 0., 0.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(0., -10., 0.), Vector3::new(0., 1., 0.));
        assert_eq!(
            Some((Point3::new(0., -5., 0.), vec3(0., -1., 0.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(0., 10., 0.), Vector3::new(0., -1., 0.));
        assert_eq!(
            Some((Point3::new(0., 5., 0.), vec3(0., 1., 0.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(0., 0., -10.), Vector3::new(0., 0., 1.));
        assert_eq!(
            Some((Point3::new(0., 0., -5.), vec3(0., 0., -1.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(0., 0., 10.), Vector3::new(0., 0., -1.));
        assert_eq!(
            Some((Point3::new(0., 0., 5.), vec3(0., 0., 1.))),
            cuboid.intersection_normal(&ray)
        );
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(None, cuboid.intersection_normal(&ray));
    }

    #[test]
    fn test_ray_continuous_transformed() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 1., 0., 0.);
        assert_eq!(
            Some(Point3::new(5., 0., 0.)),
            cuboid.intersection_transformed(&ray, &t)
        );
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(None, cuboid.intersection_transformed(&ray, &t));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 0., 0., 0.3);
        let p = cuboid.intersection_transformed(&ray, &t).unwrap();
        assert_ulps_eq!(5.233758, p.x);
        assert_ulps_eq!(0., p.y);
        assert_ulps_eq!(0., p.z);
    }

    #[test]
    fn test_ray_continuous_normal_transformed() {
        let cuboid = Cuboid::new(10., 10., 10.);
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 1., 0., 0.);
        assert_eq!(
            Some((Point3::new(5., 0., 0.), vec3(1., 0., 0.))),
            cuboid.intersection_normal_transformed(&ray, &t)
        );
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(None, cuboid.intersection_normal_transformed(&ray, &t));
        let ray = Ray3::new(Point3::new(10., 0., 0.), Vector3::new(-1., 0., 0.));
        let t = transform(0., 0., 0., 0.3);
        let p = cuboid.intersection_normal_transformed(&ray, &t).unwrap();
        assert_ulps_eq!(5.233758, p.0.x);
        assert_ulps_eq!(0., p.0.y);
        assert_ulps_eq!(0., p.0.z);
        assert_ulps_eq!(0.9553365, p.1.x);
        assert_ulps_eq!(0.2955203, p.1.y);
        assert_ulps_eq!(0., p.1.z);
    }

    // util
    fn transform(dx: f32, dy: f32, dz: f32, rot: f32) -> Decomposed<Vector3<f32>, Quaternion<f32>> {
        Decomposed {
            scale: 1.,
            rot: Quaternion::from_angle_z(Rad(rot)),
            disp: Vector3::new(dx, dy, dz),
        }
    }

    fn bound(min_x: f32, min_y: f32, min_z: f32, max_x: f32, max_y: f32, max_z: f32) -> Aabb3<f32> {
        Aabb3::new(
            Point3::new(min_x, min_y, min_z),
            Point3::new(max_x, max_y, max_z),
        )
    }
}
